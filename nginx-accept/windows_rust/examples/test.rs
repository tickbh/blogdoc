extern crate psocket;
extern crate winapi;
extern crate subprocess;

use subprocess::{Popen, PopenConfig, Redirection};

use psocket::{TcpSocket, SOCKET, INVALID_SOCKET};
use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::io::ErrorKind;
use std::mem;
use std::ptr::null_mut;
use std::ffi::OsStr;
use std::iter::once;

use std::os::windows::ffi::OsStrExt;
use winapi::um::processthreadsapi::{GetCurrentProcess, CreateProcessW, LPPROCESS_INFORMATION, PROCESS_INFORMATION, STARTUPINFOW};
use winapi::shared::minwindef::{FALSE, TRUE, DWORD, HINSTANCE__, LPVOID};
use winapi::um::winbase::{CREATE_SUSPENDED, DETACHED_PROCESS, CREATE_NEW_PROCESS_GROUP, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE, STARTF_USESTDHANDLES, CREATE_UNICODE_ENVIRONMENT};
use winapi::um::winnt::{HANDLE, PROCESS_ALL_ACCESS, DUPLICATE_SAME_ACCESS};
use winapi::um::processenv::{GetStdHandle};
use winapi::um::handleapi::{DuplicateHandle, SetHandleInformation, CloseHandle};


fn createSubProcess(arg: String) {

    let mut hStdInRead = 0 as HANDLE; //子进程用的stdin的读入端  
    let mut hStdInWrite: HANDLE = unsafe { GetStdHandle(STD_INPUT_HANDLE) }; //主程序用的stdin的读入端 

    let mut hStdOutRead = 0 as HANDLE; //主程序+用的stdout的读入端  
    let mut hStdOutWrite: HANDLE = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }; //子进程用的stdout的写入端  

    let mut hStdErrWrite: HANDLE = unsafe { GetStdHandle(STD_ERROR_HANDLE) }; //子进程用的stderr的写入端  

    const HANDLE_FLAG_INHERIT: DWORD = 0x00000001;
    
    let mut proc_info = PROCESS_INFORMATION {
        hProcess: null_mut(),
        hThread: null_mut(),
        dwProcessId: 0,
        dwThreadId: 0,
    };
    
    let mut startup_info : STARTUPINFOW = unsafe { mem::zeroed() };
    startup_info.cb = mem::size_of::<STARTUPINFOW>() as DWORD;

    let mut arg: Vec<u16> = OsStr::new(&arg).encode_wide().chain(once(0)).collect();
    let mut legit_proc = unsafe { 
                        CreateProcessW (null_mut(),
                                        arg.as_mut_ptr(),
                                        null_mut(), null_mut(), TRUE,
                                        CREATE_UNICODE_ENVIRONMENT,
                                        null_mut(), null_mut(),
                                        &mut startup_info, &mut proc_info)  
    };

    unsafe {
        CloseHandle(proc_info.hThread);
    }
}

fn main() {

    let args = ::std::env::args();
    let mut list = vec![];
    for arg in args {
        list.push(arg.to_string());
    }
    let exec_name = list[0].clone();
    println!("exec_name!!!!!!!!! {:?}", exec_name);

    TcpSocket::new_v4().unwrap();

    let mut socket: SOCKET = INVALID_SOCKET;
    let mut is_sub = false;
    if list.len() > 1 {
        is_sub = true;
        socket = list[1].parse::<SOCKET>().unwrap();
    }

    let mut proc_info = PROCESS_INFORMATION {
        hProcess: null_mut(),
        hThread: null_mut(),
        dwProcessId: 0,
        dwThreadId: 0,
    };

    let mut startup_info : STARTUPINFOW = unsafe { mem::zeroed() };
	startup_info.cb = mem::size_of::<STARTUPINFOW>() as DWORD;
	
    let start = Instant::now();
	if !is_sub {
        let listener = TcpSocket::new_v4().unwrap();
        listener.set_reuse_addr().unwrap();
        listener.bind_exist("0.0.0.0:1234").unwrap();

        println!("listener = {:?}", listener);

        // let mut p = Popen::create(&[exec_name.clone(), format!("{:?}", listener.as_raw_socket())], PopenConfig {
        //     ..Default::default()
        //     // stdout: Redirection::Pipe, stderr: Redirection::Pipe, ..Default::default()
        // }).unwrap();

        //     let ten_millis = ::std::time::Duration::from_millis(1000);
        //     ::std::thread::sleep(ten_millis);

        // let mut p = Popen::create(&[exec_name.clone(), format!("{:?}", listener.as_raw_socket())], PopenConfig {
        //     ..Default::default()
        //     // stdout: Redirection::Pipe, stderr: Redirection::Pipe, ..Default::default()
        // }).unwrap();


        // // Since we requested stdout to be redirected to a pipe, the parent's
        // // end of the pipe is available as p.stdout.  It can either be read
        // // directly, or processed using the communicate() method:
        // let (out, err) = p.communicate(None)?;

        // // check if the process is still alive
        // if let Some(exit_status) = p.poll() {
        // // the process has finished
        // } else {
        // // it is still running, terminate it
        //     p.terminate()?;
        // }


        let clone_listener = listener.try_clone().unwrap();
        let mut arg = exec_name.clone() + &format!(" {:?}", clone_listener.as_raw_socket());
        createSubProcess(arg.clone());
        createSubProcess(arg);

        // let mut arg: Vec<u16> = OsStr::new(&arg).encode_wide().chain(once(0)).collect();
        // let mut legit_proc = unsafe { 
        //                     CreateProcessW (null_mut(),
        //                                     arg.as_mut_ptr(),
        //                                     //mal_path.as_mut_ptr(),
        //                                     null_mut(), null_mut(), TRUE,
        //                                     //Create thread in suspended state
        //                                     // 0x00000004,
        //                                     0x08000000,
        //                                     // CREATE_SUSPENDED,
        //                                     null_mut(), null_mut(),
        //                                     &mut startup_info, &mut proc_info)  
        // };

        // let mut arg = exec_name.clone() + &format!(" {:?}", listener.as_raw_socket());
        // let mut arg: Vec<u16> = OsStr::new(&arg).encode_wide().chain(once(0)).collect();
        // let mut legit_proc = unsafe { 
        //                     CreateProcessW (null_mut(),
        //                                     arg.as_mut_ptr(),
        //                                     //mal_path.as_mut_ptr(),
        //                                     null_mut(), null_mut(), TRUE,
        //                                     //Create thread in suspended state
        //                                     // 0x00000004,
        //                                     0x08000000,
        //                                     // CREATE_SUSPENDED,
        //                                     null_mut(), null_mut(),
        //                                     &mut startup_info, &mut proc_info)  
        // };

        //     for stream in clone_listener.incoming() {
        //     match stream {
        //         Ok(stream) => {
        //             println!("{:?} server receive a new client! {:?}", start, stream);
        //             ::std::mem::forget(stream);

        //             // let ten_millis = ::std::time::Duration::from_millis(1000);
        //             // ::std::thread::sleep(ten_millis);
        //         }
        //         Err(e) => { /* connection failed */ }
        //     }
        // }


        // let mut legit_proc = unsafe { 
        //                     CreateProcessW (null_mut(),
        //                                     exec_name.as_mut_ptr(),
        //                                     //mal_path.as_mut_ptr(),
        //                                     null_mut(), null_mut(), FALSE,
        //                                     //Create thread in suspended state
        //                                     // 0x00000004,
        //                                     0x00000010,
        //                                     // CREATE_SUSPENDED,
        //                                     null_mut(), null_mut(),
        //                                     &mut startup_info, &mut proc_info)  
        // };

        // drop(clone_listener);
        // drop(listener);
        // println!("legit_proc = {:?}", legit_proc);
        // println!("proc_info = {:?}", proc_info);
        loop {
            let ten_millis = ::std::time::Duration::from_millis(1000);
            ::std::thread::sleep(ten_millis);
        }
    } else {
        let listener = TcpSocket::new_by_fd(socket).unwrap();
        println!("listener = {:?}", listener);
        println!("start bind 0.0.0.0:1234");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("{:?} server receive a new client! {:?}", start, stream);
                    ::std::mem::forget(stream);
                }
                Err(e) => { /* connection failed */ }
            }
        }
    }
}