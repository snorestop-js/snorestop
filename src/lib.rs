#![feature(asm, naked_functions)]

use winapi::um::consoleapi::AllocConsole;
use winapi::um::wincon::AttachConsole;

mod version;
#[macro_use]
mod macros;

// use macros::convert_to_wide;

#[no_mangle]
pub unsafe extern "stdcall" fn DllMain(
    _hinst_dll: winapi::shared::minwindef::HINSTANCE,
    fdw_reason: u32,
    _: *mut winapi::ctypes::c_void,
) {
    // File::create("called dllmain {}", fdw_reason);
    if fdw_reason == winapi::um::winnt::DLL_PROCESS_ATTACH {
        // printbox!("woo", "process attached");
        entrypoint();
    } else if fdw_reason == winapi::um::winnt::DLL_PROCESS_DETACH {
        // printbox!("sad", "process detached");
    }
}

fn entrypoint() {
    unsafe {
        if std::env::current_exe().expect("how").file_name().expect("no???") == "Among Us.exe" {
            AllocConsole();
            return;
        }
        version::initialize();
    }

    println!("sad");
}