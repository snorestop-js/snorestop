use std::ffi::{c_void, CString};
use std::mem::size_of;
use std::os::raw::c_char;
use std::ptr::null_mut;

use winapi::shared::minwindef::{FARPROC, HMODULE, LPVOID};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::winnt::{IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_IMPORT_DESCRIPTOR, LPCSTR, PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS, PAGE_READWRITE};
use winapi::um::memoryapi::VirtualProtect;

macro_rules! hook {
    ($module: expr, $target: literal, $target_func: expr, $detour_func: ident) => {
        iat_hook($module, $target, $target_func as *mut c_void, $detour_func as *mut c_void)
    };
}

unsafe extern "C" fn get_proc_address_detour(dll: HMODULE, name: LPCSTR) -> FARPROC {
    GetProcAddress(dll, name)
}

unsafe fn iat_hook(dll: HMODULE, target: &str, target_func: *mut c_void, detour_func: *mut c_void) -> bool {
    let mz = dll as PIMAGE_DOS_HEADER;
    let nt = (mz.add((*mz).e_lfanew as usize)) as PIMAGE_NT_HEADERS;
    let mut imports = mz.add((*nt).OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize].VirtualAddress as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
    println!("{}", *((*imports).Name as *mut char));
    let target = CString::new(target).expect("failed to convert target name to CString");

    println!("A");

    loop {
        println!("x {:?}", imports);
        let import = *imports;
        if *import.u.Characteristics() == 0 {
            break;
        }

        println!("B");

        if CString::from_raw(mz.add(import.Name as usize) as *mut c_char) != target {
            imports = imports.add(size_of::<IMAGE_IMPORT_DESCRIPTOR>());
            continue;
        }
        println!("C");

        let mut thunk = mz.add(import.FirstThunk as usize) as *mut *mut c_void;

        loop {
            let import = *thunk;
            if *thunk == 0 as *mut c_void {
                break;
            }
            if import != target_func {
                thunk = thunk.add(size_of::<*mut c_void>());
                continue;
            }
            println!("{:?}", thunk);
            //we got the target
            let mut old_state = 0;
            if VirtualProtect(thunk as LPVOID, size_of::<*mut c_void>(), PAGE_READWRITE, &mut old_state) == 0 {
                return false;
            }
            //successfully made memory rw
            *thunk = detour_func;
            VirtualProtect(thunk as LPVOID, size_of::<*mut c_void>(), old_state, &mut old_state);
        }
    }
    false
}

#[no_mangle]
pub extern "C" fn entrypoint() {
    unsafe {
        //i'm fucking copying doorstop code help
        /*std::thread::spawn(||*/ if std::env::current_exe().expect("how").file_name().expect("no???") == "Among Us.exe" {
            AllocConsole();

            println!("Getting module");
            let mut target_module = GetModuleHandleA(CString::new("UnityPlayer").unwrap().into_raw());
            if target_module == null_mut() {
                println!("sussy");
                target_module = GetModuleHandleA(null_mut());
            }
            println!("Using module at {:?}", target_module);
            if hook!(target_module, "kernel32.dll", GetProcAddress, get_proc_address_detour) {
                println!("Hooked GetProcAddress! Snorestop starting up...");
            } else {
                eprintln!("Failed to hook GetProcAddress!!!");
            }

            return;
        }//);
    }
}