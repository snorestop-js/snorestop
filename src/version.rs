#![allow(non_upper_case_globals)]

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStringExt;

use paste::paste;
use winapi::shared::minwindef::FARPROC;
use winapi::um::libloaderapi::LoadLibraryW;
use winapi::um::sysinfoapi::GetSystemDirectoryW;

use crate::{gen_version_func, version_func, wstr_convert, printbox};

gen_version_func!(GetFileVersionInfoA);
gen_version_func!(GetFileVersionInfoByHandle);
gen_version_func!(GetFileVersionInfoExW);
gen_version_func!(GetFileVersionInfoExA);
gen_version_func!(GetFileVersionInfoSizeA);
gen_version_func!(GetFileVersionInfoSizeExA);
gen_version_func!(GetFileVersionInfoSizeExW);
gen_version_func!(GetFileVersionInfoSizeW);
gen_version_func!(GetFileVersionInfoW);
gen_version_func!(VerFindFileA);
gen_version_func!(VerFindFileW);
gen_version_func!(VerInstallFileA);
gen_version_func!(VerInstallFileW);
gen_version_func!(VerLanguageNameA);
gen_version_func!(VerLanguageNameW);
gen_version_func!(VerQueryValueA);
gen_version_func!(VerQueryValueW);

pub unsafe fn initialize() {
    let mut data= [0u16; 255];
    let actual_len = GetSystemDirectoryW(data.as_mut_ptr(), 255);
    let mut data = OsString::from_wide(&data[..(actual_len as usize)]);
    data.push(OsStr::new("\\version.dll"));
    std::fs::write("book", format!("{:?}", data)).expect("sussy");
    let module = LoadLibraryW(wstr_convert!(data.as_os_str().to_str().unwrap()));

    printbox!("suckers", format!("uwu!! {:08X}", module as usize).as_str());

    let addr = winapi::um::libloaderapi::GetProcAddress(module, "VerFindFileA".as_ptr() as *const i8);
    printbox!("AAAAAAAAA", format!("owo {:08X}", addr as usize).as_str());

    version_func!(module, GetFileVersionInfoA);
    version_func!(module, GetFileVersionInfoByHandle);
    version_func!(module, GetFileVersionInfoExW);
    version_func!(module, GetFileVersionInfoExA);
    version_func!(module, GetFileVersionInfoSizeA);
    version_func!(module, GetFileVersionInfoSizeExA);
    version_func!(module, GetFileVersionInfoSizeExW);
    version_func!(module, GetFileVersionInfoSizeW);
    version_func!(module, GetFileVersionInfoW);
    version_func!(module, VerFindFileA);
    version_func!(module, VerFindFileW);
    version_func!(module, VerInstallFileA);
    version_func!(module, VerInstallFileW);
    version_func!(module, VerLanguageNameA);
    version_func!(module, VerLanguageNameW);
    version_func!(module, VerQueryValueA);
    version_func!(module, VerQueryValueW);
}