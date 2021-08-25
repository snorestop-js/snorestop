#include <stdio.h>
#include <stdlib.h>
#include "version.h"
#include "hook.h"

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

typedef void (* init)(HMODULE game, FARPROC init_il2cpp);

typedef void (* iat_load)(HMODULE module, const wchar_t* name);

HMODULE snorestop;
init init_entrypoint;
iat_load iat_snorestop_load;
static BOOL initialized = FALSE;

void* WINAPI get_proc_address_detour(HMODULE module, char const* name) {
    if (lstrcmpA(name, "il2cpp_init") == 0) {
        FARPROC proc = GetProcAddress(module, name);
        init_entrypoint(module, proc);
        return (void*) proc;
    }
    return (void*) GetProcAddress(module, name);
}

HMODULE WINAPI load_library_node_detour(wchar_t const* name) {
    HMODULE library = LoadLibraryW(name);
//    MessageBeep(MB_OK);
//    MessageBoxW(NULL, name, L"OK", MB_OK);
//    iat_snorestop_load(library, name);
    return library;
}

DWORD WINAPI Load(HMODULE module) {
    char* data = (char*) malloc(MAX_PATH);
    GetModuleFileNameA(NULL, data, MAX_PATH);
    if (strstr(data, "Among Us.exe")) {
        snorestop = LoadLibraryA("snorestop.dll");
        if (!snorestop) {
            MessageBoxA(NULL, "Error while loading snorestop\nReason: invalid dll", "snorestop", MB_OK);
            exit(1);
        }
//        typedef BOOL (*hook_func)(HMODULE dll, const char* target, void* target_function, void* detour_function);
        init_entrypoint = (init) GetProcAddress(snorestop, "entrypoint");
        if (!init_entrypoint) {
            MessageBoxA(NULL, "Error while loading snorestop\nReason: failed to get entrypoint", "snorestop", MB_OK);
            exit(1);
        }

//        iat_snorestop_load = (iat_load) GetProcAddress(snorestop, "iat_load");
//        if (!iat_snorestop_load) {
//            MessageBoxA(NULL, "Error while loading snorestop\nReason: failed to get iat_load", "snorestop", MB_OK);
//            exit(1);
//        }

        HMODULE target_module = GetModuleHandleA("UnityPlayer");
        const HMODULE app_module = GetModuleHandleA(NULL);

        if (!target_module) {
            target_module = app_module;
        }

        if (!iat_hook(target_module, "kernel32.dll", &GetProcAddress, &get_proc_address_detour)/* ||
            !iat_hook(snorestop, "kernel32.dll", &LoadLibraryW, &load_library_node_detour)*/) {
            MessageBoxA(NULL, "Error while loading snorestop\nReason: failed to hook GetProcAddress", "snorestop",
                        MB_OK);
            exit(1);
        }
    }
    return 0;
}

BOOL APIENTRY DllMain(HMODULE hModule, DWORD ul_reason_for_call, LPVOID lpReserved) {
    switch (ul_reason_for_call) {
        case DLL_PROCESS_ATTACH:
            DisableThreadLibraryCalls(hModule);
            load_version();
//            CreateThread(NULL, 0, (LPTHREAD_START_ROUTINE)Load, hModule, NULL, NULL);
            Load(hModule);
            break;
        case DLL_PROCESS_DETACH:
            FreeLibrary(version_dll);
            break;
    }
    return TRUE;
}