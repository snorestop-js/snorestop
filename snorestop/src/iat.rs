//iat hooking for .node files

use winapi::shared::minwindef::HMODULE;
use std::os::raw::c_char;
use winapi::um::winnt::{PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS, IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_IMPORT_DESCRIPTOR, IMAGE_DATA_DIRECTORY};
use std::ffi::CStr;
use winapi::ctypes::wchar_t;

#[no_mangle]
pub unsafe extern "C" fn iat_load(module: PIMAGE_DOS_HEADER, name: *const wchar_t) {
    // let nt = (module as usize + (*module).e_lfanew as usize) as PIMAGE_NT_HEADERS;
    // let mut imports = (module as usize + ((*nt).OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize]).VirtualAddress as usize) as *mut IMAGE_IMPORT_DESCRIPTOR;
    // loop {
    //     println!("import name {} for {}", CStr::from_ptr((module as usize + (*imports).Name as usize) as *const c_char).to_string_lossy(), CStr::from_ptr(name).to_string_lossy());
    //     imports = imports.add(1);
    // }
}
