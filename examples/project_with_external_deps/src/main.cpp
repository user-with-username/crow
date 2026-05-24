#include <stdio.h>
#include <text_formatter.h>

#ifdef _WIN32
#include <windows.h>
#endif

int main() {
  print_smt("world");

#ifdef _WIN32
  MessageBoxA(NULL, "Hello from WinAPI!", "WinAPI Test", MB_OK);

  WSADATA wsaData;
  if (WSAStartup(MAKEWORD(2, 2), &wsaData) == 0)
    WSACleanup();
#endif

  return 0;
}
