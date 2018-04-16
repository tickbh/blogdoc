# nginx源码分析-多进程socket的处理

    这篇文章主要分析的是linux及windows的socket处理，如何避免惊群及进程间负载均衡的探讨，
    这里的惊群主要是指多进程对于新建的连接如何避免同时争用accept现象的处理。

### 进程的创建
* linux
    > 进程创建的方式主要通过fork来创建出子进程

    ```c
    // src/os/unix/ngx_process.c
    ngx_pid_t ngx_spawn_process(ngx_cycle_t *cycle, ngx_spawn_proc_pt proc, void *data,
    char *name, ngx_int_t respawn) {
        ...
        pid = fork();
        ...
    }
    ```
* windows
    > 进程创建的方式主要通过CreateProcess来创建出子进程，并且通过非继承的方式创建子进程（即子进程不共享父进程的文件句柄）。

    ```c
    // src/os/win32/ngx_process.c
    ngx_pid_t ngx_execute(ngx_cycle_t *cycle, ngx_exec_ctx_t *ctx) {
        ...
        if (CreateProcess(ctx->path, ctx->args,
                      NULL, NULL, 0, //此变量为0表示句柄不继承
                      CREATE_NO_WINDOW, NULL, NULL, &si, &pi)
            == 0)
        {
            ngx_log_error(NGX_LOG_CRIT, cycle->log, ngx_errno,
                        "CreateProcess(\"%s\") failed", ngx_argv[0]);
            return 0;
        }
        ...
    }
   ```

### ListenSocket的建立
* linux
    > 由主进程先监听端口, 监听完后fork新的子进程共享父进程的socket句柄，所以在linux中，同个地址只会监听一次。在执行reload的时候会检查新的监听，或者挪除旧的监听（但如果是同一个端口的，假设127.0.0.1:80，改成0.0.0.0:80则无法生效），然后启动新的进程，同时向旧的进程发送退出状态，此时旧的进程不再接受新的连接。
    ![启动三个linux进程, 但其中只有一个监听](https://raw.githubusercontent.com/tickbh/blogdoc/master/nginx-accept/linux_nginx_process_accept.jpg)
    (启动三个linux进程, 但其中只有一个监听)
    
* windows
    > 由于不共享父进程的句柄，每个子进程都是相对独立的各体，每个进程都独立进行监听（采用的设置SO_REUSEADDR从而实现对同一个地址多次绑定的效果）。但在windows上实测，采用SO_REUSEADDR实现的监听同一个地址，只会在第一个进程能成功调用Accept函数，只有第一个进程被关闭后，第二个监听到才能成功Accept。
    ![启动8个进程, 每个程序都重复监听了该端口](https://raw.githubusercontent.com/tickbh/blogdoc/master/nginx-accept/windows_listen_88.jpg)
    (启动8个进程, 每个程序都重复监听了该端口)
    ![这是显示刚初始运行的情况](https://raw.githubusercontent.com/tickbh/blogdoc/master/nginx-accept/windows_init_success.jpg)
    (这是显示刚初始运行的情况)
    ![用ab测试进行的压力测试](https://raw.githubusercontent.com/tickbh/blogdoc/master/nginx-accept/windows_test_connection.jpg)
    (用ab测试进行的压力测试, 显示只有一个进程正在对外服务, 其实的都是空闲状态)

### 如何控制accept
* linux
    > 主要通过共享锁，只有得到锁的进程才会进行尝试调用accept事件

    ```c
    // src/event/ngx_event.c
    void
    ngx_process_events_and_timers(ngx_cycle_t *cycle)
    {
        //是否启用共享锁控制，linux默认启动
        if (ngx_use_accept_mutex) {
            //每次accept成功后都会重新赋该值，如果负载高，这值为正
            //从而减少负载高的进程得到锁的概率
            if (ngx_accept_disabled > 0) {
                ngx_accept_disabled--;
            } else {
                //尝试获取共享锁，该函数立即返回不等待
                //如果成功获取该锁，则进行accept事件的投递
                if (ngx_trylock_accept_mutex(cycle) == NGX_ERROR) {
                    return;
                }
            }
        }
    }
    ````
    > 当某进程连接数超过总worker_connections的7/8的时候，开始进行压力控制

    ```c
    // src/event/ngx_event_accept.c
    void ngx_event_accept(ngx_event_t *ev)
    {
        ...
        //ngx_cycle->connection_n表示当前配置总的work_connections
        //ngx_cycle->free_connection_n表示剩余可接受的连接数
        //当可用连接数越少时，ngx_accept_disabled值越大，也就是获取锁的难度越高
        ngx_accept_disabled = ngx_cycle->connection_n / 8
                            - ngx_cycle->free_connection_n;
        ...
    }
    ```
    
* windows
    > windows每个进程都是独立控制accept接收，没有锁控制，由于实测没有进程的压力都在单一的进程上(windows10测试)。

### 其它可行性方案探讨
* linux
    > linux通过启用SO_REUSEADDR及SO_REUSEPORT，达到可同一个地址在多个进程监听多次，统一由系统来分配socket给谁accept。
    优点：避免使用锁，统一系统分配
    缺点：进程负载分配不像手动控制那么精准，如果系统上有其它程序，可以通过监听同个端口达到偷取数据的目的，低版本的linux不支持此选项
    示例参考：[Linux ReusePort, ReuseAddr](https://raw.githubusercontent.com/tickbh/blogdoc/master/nginx-accept/linux_reuseaddr.c)
* windows
    > windows通过CreateProcess并且设置其中的子进程继承，通过命令行的方式把句柄值传递给子进程，启动进程后关闭主进程的句柄，从而使子进程拥有各自独立的accept权限。通过锁或者时序来控制谁来accept。
    示例参与：[Rust版的windows CreateProcess控制](https://github.com/tickbh/blogdoc/tree/master/nginx-accept/windows_rust)
    ![用ab测试进行的压力测试](https://raw.githubusercontent.com/tickbh/blogdoc/master/nginx-accept/windows_createprocess_sub.jpg)
    (运行截图, 其中584进程每接受一个新的socket时sleep 10秒时间)