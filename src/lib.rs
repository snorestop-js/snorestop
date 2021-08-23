use winapi::um::consoleapi::AllocConsole;

#[no_mangle]
pub extern "C" fn entrypoint() {
    unsafe {
        /*std::thread::spawn(||*/ if std::env::current_exe().expect("how").file_name().expect("no???") == "Among Us.exe" {
            AllocConsole();

            println!("Sex");
            let (tx, rx) = std::sync::mpsc::sync_channel::<String>(0);
            println!("Sex");
            let channel = nodejs::channel();
            println!("Sex");
            channel.send(move |mut cx| {
                use nodejs::neon::{context::Context, reflect::eval, types::JsString};
                let script = cx.string("require('http').STATUS_CODES[418]");
                let whoami = eval(&mut cx, script)?;
                let whoami = whoami.downcast_or_throw::<JsString, _>(&mut cx)?;
                tx.send(whoami.value(&mut cx)).unwrap();
                Ok(())
            });
            let whoami = rx.recv().unwrap();
            println!("{}", whoami);

            return;
        }//);
    }
}