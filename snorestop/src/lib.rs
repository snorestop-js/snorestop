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
use std::io::{self, Write};
use std::env;

static_detour! {
  static Il2cppInitDetour: unsafe extern "C" fn(*const c_char) -> bool;
}

type Il2cppInit = unsafe extern "C" fn(*const c_char) -> bool;

fn __handle_stdout(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    if cx.len() > 0 {
        let arg: Handle<JsString> = cx.argument(0).unwrap();
        let text: Handle<JsString> = arg.downcast_or_throw(&mut cx).unwrap();
        io::stdout().write_all(text.value(&mut cx).as_bytes());
    }
    Ok(JsUndefined::new(&mut cx))
}

fn __handle_stderr(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    if cx.len() > 0 {
        let arg: Handle<JsString> = cx.argument(0).unwrap();
        let text: Handle<JsString> = arg.downcast_or_throw(&mut cx).unwrap();
        io::stdout().write_all(text.value(&mut cx).as_bytes());
    }
    Ok(JsUndefined::new(&mut cx))
}

fn il2cpp_init(domain_name: *const c_char) -> bool {
    unsafe {
        Il2cppInitDetour.disable().expect("failed to disable hook");
        let output = Il2cppInitDetour.call(domain_name);
        Il2cppInitDetour.enable().expect("failed to re-enable hook");

        //il2cpp has initialized by this point :)

        {
            let channel = nodejs::channel();
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            channel.send(move |mut cx| {
                let string = JsString::new(&mut cx, "
                  process.stderr.write = (buffer) => {
                    __handleStderr(Buffer.from(buffer).toString());
                    return true
                  };
                  
                  process.stdout.write = (buffer) => {
                    __handleStdout(Buffer.from(buffer).toString());
                    return true 
                  };
                  
                  try { 
                    const fs = require(\"fs\");

                    const { Snorestop } = require('snorestop-module')
                    const snorestop = new Snorestop();
                    const selfPackageJson = JSON.parse(fs.readFileSync(path.join(__amongus_dirname, \"package.json\"), \"utf-8\"));

                    const modules = Object.keys(selfPackageJson.dependencies)
                      .map(packageName => require.resolve(packageName + \"/package.json\"))
                      .map(packageJsonPath => JSON.parse(fs.readFileSync(packageJsonPath, \"utf-8\")))
                      .filter(packageJson => packageJson[\"is-snorestop-package\"])
                      .forEach(packageJson => {
                        snorestop.load(packageJson, require.resolve(packageJson.name));
                      })

                  } catch(err) { 
                    console.error(err)
                  }");
                let js_handle_stdout_string = JsString::new(&mut cx, "__handleStdout");
                let js_handle_stderr_string = JsString::new(&mut cx, "__handleStderr");
                let js_dirname_key_string = JsString::new(&mut cx, "__amongus_dirname");
                
                let mut dirname = match env::current_exe() {
                  Err(e) => return Ok(()),
                  Ok(f) => f,
                };

                let js_dirname_string = JsString::new(&mut cx, dirname.parent().unwrap().to_str().unwrap());
                let js_handle_stdout = JsFunction::new(&mut cx, __handle_stdout).expect("failed to create stdout handler function");
                let js_handle_stderr = JsFunction::new(&mut cx, __handle_stderr).expect("failed to create stderr handler function");
                cx.global().set(&mut cx, js_handle_stdout_string, js_handle_stdout).expect("failed to set stdout handler global");
                cx.global().set(&mut cx, js_handle_stderr_string, js_handle_stderr).expect("failed to set stderr handler global");
                cx.global().set(&mut cx, js_dirname_key_string, js_dirname_string).expect("failed to set stderr handler global");
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
