use libloading::Symbol;
use std::sync::Mutex;


#[macro_export]
macro_rules! gen_version {
    ($name:ident) => {
        paste! {
            // static mut [<Original $name>]: FARPROC = 0 as FARPROC;

            extern "C" {
                static mut [<Original $name>]: FARPROC;
            }

            #[no_mangle]
            #[naked]
            extern "C" fn $name() {
                unsafe {
                    asm! {
                       "jmp [{addr}]",
                        addr = sym [<Original $name>],
                        options(noreturn)
                    };
                    // if [<Original $name>].is_some() {
                    //     // if proc as usize == 0 {
                    //     //     printbox!("Error!", format!("{} was null!", stringify!([<Original $name>])).as_str());
                    //     //     return;
                    //     // }
                    //     // let addr = [<Original $name>].unwrap().clone().into_raw().into_raw();
                    //     let addr = [<Original $name>].unwrap();
                    // } else {
                    //     printbox!("Error!", format!("{} was not set!", stringify!([<Original $name>])).as_str());
                    // }
                };
            }
        }
    };
}

#[macro_export]
macro_rules! version_func {
    ($module: expr, $name: ident) => {
        paste! {
            let addr: libloading::Symbol<FARPROC> = ($module).get(stringify!($name).as_bytes()).expect("problem in neverland");
            [<Original $name>] = (addr.into_raw().into_raw());
            // printbox!("woo", format!("{:08X} {}", [<Original $name>] as usize, stringify!($name)).as_str());
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