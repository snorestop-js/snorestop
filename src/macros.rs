#[macro_export]
macro_rules! gen_version_func {
    ($name:ident) => {
        paste! {
            static mut [<Original $name>]: Option<FARPROC> = None;

            #[no_mangle]
            extern "stdcall" fn $name() {
                unsafe {
                    if let Some(proc) = [<Original $name>] {
                        let t = proc;
                        asm! {
                           "jmp [{t}]",
                            t = in(reg) t
                        }
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! version_func {
    ($module: expr, $name: ident) => {
        paste! {
            let addr = winapi::um::libloaderapi::GetProcAddress($module, stringify!($name).as_ptr() as *const i8);
            [<Original $name>] = Some(addr);
            printbox!(stringify!($name), format!("{:08X}", addr as usize).as_str());
        }
    };
}

pub fn convert_to_wide(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect()
}
#[macro_export]
macro_rules! wstr_convert {
    ($text: expr) => {
        crate::macros::convert_to_wide($text).as_ptr()
    };
}
#[macro_export]
macro_rules! printbox {
    ($title: expr, $message: expr) => {
        winapi::um::winuser::MessageBoxW(0 as winapi::shared::windef::HWND, wstr_convert!($message), wstr_convert!($title), winapi::um::winuser::MB_OK);
    };
}