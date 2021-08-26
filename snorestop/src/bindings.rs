use std::ffi::CString;
use std::mem::transmute;

use nodejs::neon::context::{Context, FunctionContext};
use nodejs::neon::handle::Handle;
use nodejs::neon::object::Object;
use nodejs::neon::result::JsResult;
use nodejs::neon::types::{JsArray, JsFunction, JsNumber, JsArrayBuffer};
use paste::paste;
use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::GetProcAddress;

macro_rules! gen_statics {
    ($($name: ident = ($($params: ty),*) -> $ret: ty),*) => {
        paste! {
            $(
                #[allow(non_upper_case_globals)]
                static mut $name: Option<fn ($($params),*) -> $ret> = None;
            )*
        }
    };
}

macro_rules! set {
    ($object: expr, $cx: expr, $name: expr, $value: expr) => {
        {
            let name = $name;
            let value = $value;
            $object.set($cx, name, value).expect(format!("failed to set value on {}", stringify!($object)).as_str())
        }
    };
}

macro_rules! get_proc {
    ($module: expr, $proc_name: ident) => {
        unsafe { $proc_name = Some(transmute(GetProcAddress($module, CString::new(stringify!($proc_name)).unwrap().into_raw()))) }
    };
}

gen_statics! {
    il2cpp_domain_get = () -> *mut u8,
    il2cpp_domain_get_assemblies = (*mut u8, *mut u8) -> *mut *mut u8,
    il2cpp_alloc = (usize) -> *mut u8
}

fn domain_get(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(unsafe { il2cpp_domain_get.unwrap()() } as i32 as f64))
}

fn domain_get_assemblies(mut cx: FunctionContext) -> JsResult<JsArray> {
    let domain: Handle<JsNumber> = cx.argument(0)?;
    let mut size = 0;
    let assemblies = unsafe { il2cpp_domain_get_assemblies.unwrap()(domain.value(&mut cx) as i32 as *mut u8, &mut size) };
    let array = cx.empty_array();

    unsafe {
        for i in 0..size {
            set!(array, &mut cx, cx.number(i as f64), cx.number((*assemblies.add(i as usize)) as i32 as f64));
        }
    }

    Ok(array)
}

fn alloc(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let size: Handle<JsNumber> = cx.argument(0)?;
    let size = size.value(&mut cx);
    if size > u32::MAX as f64 {
        panic!("Cannot allocate >4GB buffers!");
    }
    let buffer = unsafe {
        let memory = il2cpp_alloc.unwrap()(size as i32 as usize);
        std::slice::from_raw_parts_mut(memory, size as usize)
    };
    Ok(JsArrayBuffer::external(&mut cx, buffer))
}

pub(crate) fn load_functions<'a, C: Context<'a>>(module: HMODULE, cx: &mut C) {
    let global_obj = cx.empty_object();
    get_proc!(module, il2cpp_domain_get);
    get_proc!(module, il2cpp_domain_get_assemblies);
    get_proc!(module, il2cpp_alloc);
    set!(global_obj, cx, cx.string("il2cpp_domain_get"), JsFunction::new(cx, domain_get).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_domain_get_assemblies"), JsFunction::new(cx, domain_get_assemblies).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_alloc"), JsFunction::new(cx, alloc).expect("failed to create a js_function"));
    set!(cx.global(), cx, "__IL2CPP", global_obj);
}
