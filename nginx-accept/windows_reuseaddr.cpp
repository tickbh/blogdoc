#include "stdafx.h"
#include <tchar.h>		
#include <stdio.h>		
#include <winsock2.h>	
#include <windows.h>	

int _tmain(int argc, TCHAR *argv[]) {	

										
	WSADATA wsaData;					
	int iRet;							
	SOCKET soc;							
	u_short ns_port;					
	struct sockaddr_in saiServerAddr;	
	int optval = 1;						
	SOCKET acc;							
	struct sockaddr_in saiClientAddr;	
	int iClientAddrLen;					
	char *client_ip_addr_str;			
	u_short client_port;				

										
	iRet = WSAStartup(MAKEWORD(2, 2), &wsaData);	
	if (iRet) {	
		_tprintf(_T("Error!(iRet = %d.)\n"), iRet);	
		WSACleanup();	
		return -1;	

	}
	
	_tprintf(_T("WSAStartup success!\n"));	
											
	soc = WSASocket(AF_INET, SOCK_STREAM, IPPROTO_TCP, NULL, 0, 0);	
	if (soc == INVALID_SOCKET) {	
		_tprintf(_T("WSASocket Error!\n"));	
		WSACleanup();	
		return -1;	

	}
	
	_tprintf(_T("soc = %lu\n"), soc);	
	WSAHtons(soc, 8888, &ns_port);	
	_tprintf(_T("port = %04x, ns_port = %04x\n"), 8888, ns_port);	

																	
	saiServerAddr.sin_family = AF_INET;					
	saiServerAddr.sin_port = ns_port;					
	saiServerAddr.sin_addr.S_un.S_addr = INADDR_ANY;	

														
	if (setsockopt(soc, SOL_SOCKET, SO_REUSEADDR, (const char *)&optval, sizeof(optval)) == -1) {	
		_tprintf(_T("setsockopt(SO_REUSEADDR) error!\n"));	
		closesocket(soc);	
		WSACleanup();	
		return -1;	
	}

	_tprintf(_T("setsockopt(SO_REUSEADDR) success.\n"));	
															
	if (bind(soc, (struct sockaddr *)&saiServerAddr, sizeof(saiServerAddr)) == -1) {	
		_tprintf(_T("bind Error!\n"));	
		closesocket(soc);	
		WSACleanup();	
		return -1;	
	}
	
	_tprintf(_T("bind Success.\n"));	
	if (listen(soc, 1) == -1) {	
		_tprintf(_T("listen Error!\n"));	
		closesocket(soc);	
		WSACleanup();	
		return -1;	

	}
	
	_tprintf(_T("listen success.\n"));	
	iClientAddrLen = sizeof(saiClientAddr);	
	while (true) {
		acc = WSAAccept(soc, (struct sockaddr *)&saiClientAddr, &iClientAddrLen, NULL, NULL);	
		if (acc == INVALID_SOCKET) {	
			_tprintf(_T("accept Error!\n"));	
			closesocket(soc);	
			WSACleanup();	
			return -1;	
		}
		_tprintf(_T("acc = %d\n"), acc);	
		closesocket(acc);	

	}
						
	closesocket(soc);	

						
	WSACleanup();	

					
	return 0;
}