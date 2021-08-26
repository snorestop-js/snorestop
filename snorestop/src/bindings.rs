use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::os::raw::c_char;
use nodejs::neon::context::{Context, FunctionContext};
use nodejs::neon::handle::Handle;
use nodejs::neon::object::Object;
use nodejs::neon::result::JsResult;
use nodejs::neon::types::{JsArray, JsFunction, JsNumber, JsString, JsUndefined, JsArrayBuffer};
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
    il2cpp_assembly_get_image = (*mut usize) -> *mut usize,
    il2cpp_image_get_name = (*mut usize) -> *mut c_char,
    il2cpp_image_get_filename = (*mut usize) -> *mut c_char,
    il2cpp_image_get_assembly = (*mut usize) -> *mut usize,
    il2cpp_image_get_class_count = (*mut usize) -> usize,
    il2cpp_image_get_class = (*mut usize, usize) -> *mut usize,
    il2cpp_class_get_namespace = (*mut usize) -> *mut c_char,
    il2cpp_class_get_name = (*mut usize) -> *mut c_char,
    il2cpp_domain_get = () -> *mut u8,
    il2cpp_domain_get_assemblies = (*mut u8, *mut u8) -> *mut *mut u8,
    il2cpp_alloc = (usize) -> *mut u8
}

fn create_buffer(mut cx: FunctionContext, size: u32, address: Option<*mut u8>,) -> JsResult<JsArrayBuffer> {
    if let Some(address) = address {
        let array_buffer = unsafe { JsArrayBuffer::external(&mut cx, std::slice::from_raw_parts_mut(address, size as usize)) };
        set!(array_buffer, &mut cx, cx.string("ptr"), cx.number(address as usize as f64));
        Ok(array_buffer)
    } else {
        let mut buffer = &mut vec![0u8; size as usize][..];
        let ptr = buffer.as_mut_ptr();
        let array_buffer = JsArrayBuffer::external(&mut cx, &mut buffer);
        set!(array_buffer, &mut cx, cx.string("ptr"), cx.number(ptr as usize as f64));
        Ok(array_buffer)
    }
}

fn create_buffer_js(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let size: Handle<JsNumber> = cx.argument(0)?;
    let address = cx.argument_opt(1).map(|arg| {
        let number: Handle<JsNumber>  = arg.downcast_or_throw(&mut cx).unwrap();
        number.value(&mut cx) as usize as *mut u8
    });
    let size = size.value(&mut cx) as u32;
    create_buffer(cx, size, address)
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

fn assembly_get_image(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
      let domain: Handle<JsNumber> = cx.argument(0)?;
      let imagePtr = il2cpp_assembly_get_image.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
      cx.number(imagePtr as u32)
    })
}

fn image_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
      let domain: Handle<JsNumber> = cx.argument(0)?;
      let strPtr = il2cpp_image_get_name.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
      cx.string(CStr::from_ptr(strPtr).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn image_get_filename(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
      let domain: Handle<JsNumber> = cx.argument(0)?;
      let strPtr = il2cpp_image_get_filename.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
      cx.string(CStr::from_ptr(strPtr).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn image_get_assembly(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
      let domain: Handle<JsNumber> = cx.argument(0)?;
      let strPtr = il2cpp_image_get_assembly.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
      cx.number(strPtr as u32)
    })
}

fn image_get_class_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
      let domain: Handle<JsNumber> = cx.argument(0)?;
      let strPtr = il2cpp_image_get_class_count.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
      cx.number(strPtr as u32)
    })
}

fn image_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
      let domain: Handle<JsNumber> = cx.argument(0)?;
      let domain2: Handle<JsNumber> = cx.argument(1)?;
      let strPtr = il2cpp_image_get_class.unwrap()(domain.value(&mut cx) as i32 as *mut usize, domain2.value(&mut cx) as usize);
      cx.number(strPtr as u32)
    })
}

fn class_get_namespace(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
      let domain: Handle<JsNumber> = cx.argument(0)?;
      let strPtr = il2cpp_class_get_namespace.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
      cx.string(CStr::from_ptr(strPtr).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn class_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
      let domain: Handle<JsNumber> = cx.argument(0)?;
      let strPtr = il2cpp_class_get_name.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
      cx.string(CStr::from_ptr(strPtr).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn alloc(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let size: Handle<JsNumber> = cx.argument(0)?;
    let size = size.value(&mut cx);
    if size > u32::MAX as f64 {
        panic!("Cannot allocate >4GB buffers!");
    }
    let buffer = unsafe { il2cpp_alloc.unwrap()(size as i32 as usize) };
    create_buffer(cx, size as u32, Some(buffer))
}

pub(crate) fn load_functions<'a, C: Context<'a>>(module: HMODULE, cx: &mut C) {
    let global_obj = cx.empty_object();
    get_proc!(module, il2cpp_domain_get);
    get_proc!(module, il2cpp_domain_get_assemblies);
    get_proc!(module, il2cpp_assembly_get_image);
    get_proc!(module, il2cpp_image_get_name);
    get_proc!(module, il2cpp_image_get_filename);
    get_proc!(module, il2cpp_image_get_assembly);
    get_proc!(module, il2cpp_image_get_class_count);
    get_proc!(module, il2cpp_image_get_class);
    get_proc!(module, il2cpp_class_get_namespace);
    get_proc!(module, il2cpp_class_get_name);
    get_proc!(module, il2cpp_alloc);
    set!(global_obj, cx, cx.string("il2cpp_domain_get"), JsFunction::new(cx, domain_get).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_domain_get_assemblies"), JsFunction::new(cx, domain_get_assemblies).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_assembly_get_image"), JsFunction::new(cx, assembly_get_image).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_image_get_name"), JsFunction::new(cx, image_get_name).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_image_get_filename"), JsFunction::new(cx, image_get_filename).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_image_get_assembly"), JsFunction::new(cx, image_get_assembly).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_image_get_class_count"), JsFunction::new(cx, image_get_class_count).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_image_get_class"), JsFunction::new(cx, image_get_class).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_class_get_namespace"), JsFunction::new(cx, class_get_namespace).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_class_get_name"), JsFunction::new(cx, class_get_name).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_domain_get"), JsFunction::new(cx, domain_get).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_domain_get_assemblies"), JsFunction::new(cx, domain_get_assemblies).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_alloc"), JsFunction::new(cx, alloc).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("snorestop_create_buffer"), JsFunction::new(cx, create_buffer_js).expect("failed to create a js_function"));
    set!(cx.global(), cx, "__IL2CPP", global_obj);
}
