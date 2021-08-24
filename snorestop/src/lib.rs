use std::ffi::CStr;
use std::mem::transmute;
use std::os::raw::c_char;

use detour::static_detour;
use nodejs::{
    neon::{
        context::{
            Context,
            FunctionContext
        },
        object::Object,
        reflect::eval,
        types::{
            JsFunction,
            JsString,
            JsUndefined
        },
        handle::Handle,
        result::JsResult
    },
};
use winapi::shared::minwindef::{FARPROC, HMODULE};
use winapi::um::consoleapi::AllocConsole;

static_detour! {
  static Il2cppInitDetour: unsafe extern "C" fn(*const c_char) -> bool;
}

type Il2cppInit = unsafe extern "C" fn(*const c_char) -> bool;

fn __print(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    if cx.len() < 1 {
        println!();
    } else {
        let arg: Handle<JsString> = cx.argument(0).unwrap();
        let text: Handle<JsString> = arg.downcast_or_throw(&mut cx).unwrap();
        println!("{}", text.value(&mut cx))
    }
    Ok(JsUndefined::new(&mut cx))
}

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
                let key = JsString::new(&mut cx, "__print");
                let function = JsFunction::new(&mut cx, __print).expect("failed to create print function");
                cx.global().set(&mut cx, key, function).expect("failed to set print global");
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