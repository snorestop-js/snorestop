use std::ffi::CStr;
use std::os::raw::c_char;

use winapi::shared::minwindef::{HMODULE, FARPROC};
use winapi::um::consoleapi::AllocConsole;
use detour::static_detour;
use std::mem::transmute;
use nodejs::neon::{types::JsString, reflect::eval};

static_detour! {
  static Il2cppInitDetour: unsafe extern "C" fn(*const c_char) -> bool;
}

type Il2cppInit = unsafe extern "C" fn(*const c_char) -> bool;

fn il2cpp_init(domain_name: *const c_char) -> bool {
    unsafe {
        println!("OMG NO WAY LFG EZ {}", CStr::from_ptr(domain_name).to_string_lossy());
        Il2cppInitDetour.disable().expect("failed to disable hook");
        let output = Il2cppInitDetour.call(domain_name);
        Il2cppInitDetour.enable().expect("failed to reenable hook");

        //il2cpp has initialized by this point :)

        {
            let channel = nodejs::channel();
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            channel.send(move |mut cx| {
                let string = JsString::new(&mut cx, "require('./snorestop/index.js')");
                eval(&mut cx, string).unwrap();
                sender.send(()).unwrap();
                Ok(())
            });
            receiver.recv().unwrap();
        }

        output
    }
}

#[no_mangle]
pub extern "C" fn entrypoint(_assembly: HMODULE, proc: FARPROC) {
    unsafe {
        AllocConsole();
        let o: Il2cppInit = transmute(proc);
        Il2cppInitDetour.initialize(o, il2cpp_init).unwrap();
        Il2cppInitDetour.enable().expect("failed to hook il2cpp_init");
    }
}