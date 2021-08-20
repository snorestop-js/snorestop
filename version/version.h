#pragma once
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <malloc.h>

extern HMODULE version_dll;

DWORD WINAPI Load(LPVOID lpParam);