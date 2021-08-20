#![allow(non_upper_case_globals)]

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStringExt;

use paste::paste;
use winapi::shared::minwindef::FARPROC;
use winapi::um::libloaderapi::LoadLibraryW;
use winapi::um::sysinfoapi::GetSystemDirectoryW;

use crate::{gen_version, printbox, version_func, wstr_convert};
use libloading::Library;
use std::sync::{Mutex, Arc};
use std::marker::PhantomData;

// gen_version!(GetFileVersionInfoA);
// gen_version!(GetFileVersionInfoByHandle);
// gen_version!(GetFileVersionInfoExW);
// gen_version!(GetFileVersionInfoExA);
// gen_version!(GetFileVersionInfoSizeA);
// gen_version!(GetFileVersionInfoSizeExA);
// gen_version!(GetFileVersionInfoSizeExW);
// gen_version!(GetFileVersionInfoSizeW);
// gen_version!(GetFileVersionInfoW);
// gen_version!(VerFindFileA);
// gen_version!(VerFindFileW);
// gen_version!(VerInstallFileA);
// gen_version!(VerInstallFileW);
// gen_version!(VerLanguageNameA);
// gen_version!(VerLanguageNameW);
// gen_version!(VerQueryValueA);
// gen_version!(VerQueryValueW);
// static mut module: Option<Library> = None;

extern "C" {
    fn load_version();
}

pub unsafe fn initialize() {
    load_version();
    // let mut data = [0u16; 255];
    // let actual_len = GetSystemDirectoryW(data.as_mut_ptr(), 255);
    // let mut data = OsString::from_wide(&data[..(actual_len as usize)]);
    // data.push(OsStr::new("\\version.dll"));
    // std::fs::write("book", format!("{:?}", data)).expect("sussy");
    // // let module = LoadLibraryW(wstr_convert!(data.as_os_str().to_str().unwrap()));
    // let library = libloading::Library::new(data.as_os_str());
    // if let Err(err) = library {
    //     printbox!("Error!", format!("Failed to load library:\n{}", err.to_string()).as_str());
    //     return;
    // }
    // let modu = (library.unwrap());

    // printbox!("suckers", format!("uwu!! {:08X}", module.).as_str());

    // let addr: libloading::Symbol<FARPROC> = modu.get(stringify!($name).as_bytes()).expect("problem in neverland");
    // let lmao = addr.into_raw().into_raw();

    // let addr = winapi::um::libloaderapi::GetProcAddress(module, "VerFindFileA".as_mut_ptr() as *const i8);
    // printbox!("AAAAAAAAA", format!("owo {:08X}", addr as usize).as_str());
    // version_func!(modu, GetFileVersionInfoA);
    // version_func!(modu, GetFileVersionInfoByHandle);
    // version_func!(modu, GetFileVersionInfoExW);
    // version_func!(modu, GetFileVersionInfoExA);
    // version_func!(modu, GetFileVersionInfoSizeA);
    // version_func!(modu, GetFileVersionInfoSizeExA);
    // version_func!(modu, GetFileVersionInfoSizeExW);
    // version_func!(modu, GetFileVersionInfoSizeW);
    // version_func!(modu, GetFileVersionInfoW);
    // version_func!(modu, VerFindFileA);
    // version_func!(modu, VerFindFileW);
    // version_func!(modu, VerInstallFileA);
    // version_func!(modu, VerInstallFileW);
    // version_func!(modu, VerLanguageNameA);
    // version_func!(modu, VerLanguageNameW);
    // version_func!(modu, VerQueryValueA);
    // version_func!(modu, VerQueryValueW);
}