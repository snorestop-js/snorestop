#include "version.h"

HMODULE version_dll;

#define WRAPPER_GENFUNC(name) \
	FARPROC o##name; \
	__declspec(naked) void _##name() \
	{ \
		__asm jmp[o##name] \
	}

WRAPPER_GENFUNC(GetFileVersionInfoA)
WRAPPER_GENFUNC(GetFileVersionInfoByHandle)
WRAPPER_GENFUNC(GetFileVersionInfoExW)
WRAPPER_GENFUNC(GetFileVersionInfoExA)
WRAPPER_GENFUNC(GetFileVersionInfoSizeA)
WRAPPER_GENFUNC(GetFileVersionInfoSizeExA)
WRAPPER_GENFUNC(GetFileVersionInfoSizeExW)
WRAPPER_GENFUNC(GetFileVersionInfoSizeW)
WRAPPER_GENFUNC(GetFileVersionInfoW)
WRAPPER_GENFUNC(VerFindFileA)
WRAPPER_GENFUNC(VerFindFileW)
WRAPPER_GENFUNC(VerInstallFileA)
WRAPPER_GENFUNC(VerInstallFileW)
WRAPPER_GENFUNC(VerLanguageNameA)
WRAPPER_GENFUNC(VerLanguageNameW)
WRAPPER_GENFUNC(VerQueryValueA)
WRAPPER_GENFUNC(VerQueryValueW)

#define WRAPPER_FUNC(name) o##name = GetProcAddress(version_dll, ###name);

void load_version() {
    char systemPath[MAX_PATH];
    GetSystemDirectoryA(systemPath, MAX_PATH);
    strcat_s(systemPath, MAX_PATH, "\\version.dll");
    version_dll = LoadLibraryA(systemPath);

#if _DEBUG
    if (!version_dll) {
        MessageBoxA(NULL, "Unable to load version.dll", "snorestop", MB_OK | MB_ICONERROR | MB_SYSTEMMODAL);
    }
#endif

    if (!version_dll) return;

    WRAPPER_FUNC(GetFileVersionInfoA);
    WRAPPER_FUNC(GetFileVersionInfoByHandle);
    WRAPPER_FUNC(GetFileVersionInfoExW);
    WRAPPER_FUNC(GetFileVersionInfoExA);
    WRAPPER_FUNC(GetFileVersionInfoSizeA);
    WRAPPER_FUNC(GetFileVersionInfoSizeExW);
    WRAPPER_FUNC(GetFileVersionInfoSizeExA);
    WRAPPER_FUNC(GetFileVersionInfoSizeW);
    WRAPPER_FUNC(GetFileVersionInfoW);
    WRAPPER_FUNC(VerFindFileA);
    WRAPPER_FUNC(VerFindFileW);
    WRAPPER_FUNC(VerInstallFileA);
    WRAPPER_FUNC(VerInstallFileW);
    WRAPPER_FUNC(VerLanguageNameA);
    WRAPPER_FUNC(VerLanguageNameW);
    WRAPPER_FUNC(VerQueryValueA);
    WRAPPER_FUNC(VerQueryValueW);
}

BOOL APIENTRY DllMain(HMODULE hModule, DWORD ul_reason_for_call, LPVOID lpReserved)
{
    switch (ul_reason_for_call)
    {
        case DLL_PROCESS_ATTACH:
            DisableThreadLibraryCalls(hModule);
            load_version();
            char* data = (char*) malloc(MAX_PATH);
            GetModuleFileNameA(NULL, data, MAX_PATH);
            if (strstr(data, "Among Us.exe")) {
                HMODULE snorestop = LoadLibrary("snorestop.dll");
                if (!snorestop) {
                    MessageBoxA(NULL, "Failed to load snorestop.dll!", "snorestop", MB_OK);
                    return FALSE;
                }
                FARPROC entrypoint = GetProcAddress(snorestop, "entrypoint");
                if (!entrypoint) {
                    MessageBoxA(NULL, "Failed to get the entrypoint!", "snorestop", MB_OK);
                    return FALSE;
                }
                entrypoint();
            }
            break;
        case DLL_PROCESS_DETACH:
            FreeLibrary(version_dll);
            break;
    }
    return TRUE;
}