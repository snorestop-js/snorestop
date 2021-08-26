use std::borrow::BorrowMut;
use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::os::raw::{c_char, c_void};
use nodejs::neon::context::{Context, FunctionContext};
use nodejs::neon::handle::Handle;
use nodejs::neon::object::Object;
use nodejs::neon::result::JsResult;
use nodejs::neon::types::{JsArray, JsArrayBuffer, JsBoolean, JsFunction, JsNumber, JsString, JsUndefined, JsValue, Value};
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
    il2cpp_class_from_name = (*mut usize, *mut c_char, *mut c_char) -> *mut usize,
    il2cpp_assembly_get_image = (*mut c_void) -> *mut c_void,
    il2cpp_image_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_image_get_filename = (*mut c_void) -> *mut c_char,
    il2cpp_image_get_assembly = (*mut c_void) -> *mut c_void,
    il2cpp_image_get_class_count = (*mut c_void) -> usize,
    il2cpp_image_get_class = (*mut c_void, usize) -> *mut usize,
    il2cpp_class_get_namespace = (*mut c_void) -> *mut c_char,
    il2cpp_class_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_class_get_methods = (*mut c_void, *mut usize) -> *mut c_void,
    il2cpp_class_get_fields = (*mut c_void, *mut usize) -> *mut c_void,
    il2cpp_field_get_parent = (*mut c_void) -> *mut c_void,
    il2cpp_field_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_field_static_get_value = (*mut c_void, *mut u32) -> c_void,
    il2cpp_field_get_type = (*mut c_void) -> *mut c_void,
    il2cpp_type_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_type_get_type = (*mut c_void) -> usize,
    il2cpp_type_is_static = (*mut c_void) -> bool,
    il2cpp_array_length = (*mut usize) -> usize,
    il2cpp_array_get_byte_length = (*mut usize) -> usize,
    il2cpp_domain_get = () -> *mut c_void,
    il2cpp_domain_get_assemblies = (*mut c_void, *mut usize) -> *mut *mut c_void,
    il2cpp_class_get_field_from_name = (*mut usize, *mut c_char) -> *mut *mut c_void,
    il2cpp_object_get_class = (*mut usize) -> *mut c_char,
    il2cpp_alloc = (usize) -> *mut c_void
}

fn create_buffer(mut cx: FunctionContext, size: u32, address: Option<*mut c_void>) -> JsResult<JsArrayBuffer> {
    if let Some(address) = address {
        let array_buffer = unsafe {
            JsArrayBuffer::external(&mut cx, {
                let data: &mut [u8] = transmute(std::slice::from_raw_parts_mut(address, size as usize));
                data
            })
        };
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
        let number: Handle<JsNumber> = arg.downcast_or_throw(&mut cx).unwrap();
        number.value(&mut cx) as usize as *mut c_void
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
    let assemblies = unsafe { il2cpp_domain_get_assemblies.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, &mut size) };
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
        let imagePtr = il2cpp_assembly_get_image.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.number(imagePtr as u32)
    })
}

fn image_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_image_get_name.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(strPtr).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn image_get_filename(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_image_get_filename.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(strPtr).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn image_get_assembly(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_image_get_assembly.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.number(strPtr as u32)
    })
}

fn image_get_class_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_image_get_class_count.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.number(strPtr as u32)
    })
}

fn image_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let domain2: Handle<JsNumber> = cx.argument(1)?;
        let strPtr = il2cpp_image_get_class.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, domain2.value(&mut cx) as usize);
        cx.number(strPtr as u32)
    })
}

fn class_get_namespace(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_class_get_namespace.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(strPtr).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn class_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_class_get_name.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(strPtr).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn field_get_parent(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let str_ptr = il2cpp_field_get_parent.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.number(str_ptr as u32)
    })
}

fn field_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let str_ptr = il2cpp_field_get_name.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(str_ptr).to_str().expect("Failed to unwrap str_ptr"))
    })
}

fn method_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let str_ptr = il2cpp_method_get_class.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        return Ok(cx.number(str_ptr as u32));
    }
}

fn field_static_get_value(mut cx: FunctionContext) -> JsResult<JsValue> {
    unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let a_buffer: Handle<JsBoolean> = cx.argument(1)?;
        let d_size: Handle<JsNumber> = cx.argument(2)?;

        if (a_buffer.value(&mut cx)) {
            let mut resSlice = vec![0u8; d_size.value(&mut cx) as usize];
            il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, resSlice.as_mut_ptr() as *mut u32);
            return Ok(JsArrayBuffer::external(&mut cx, resSlice).as_value(&mut cx));
        } else {
            if (d_size.value(&mut cx) > 4.0 || d_size.value(&mut cx) == 3.0) {
                panic!("d_size cannot be greater than 4 or 3 when as_buffer is false");
            }

            if (d_size.value(&mut cx) == 4.0) {
                let mut res = 0;
                il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, &mut res);
                return Ok(cx.number(res).as_value(&mut cx));
            }

            if (d_size.value(&mut cx) == 2.0) {
                let mut res: i16 = 0;
                il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, &mut (res as u32));
                return Ok(cx.number(res).as_value(&mut cx));
            }

            if (d_size.value(&mut cx) == 1.0) {
                let mut res: i8 = 0;
                il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, &mut (res as u32));
                return Ok(cx.number(res).as_value(&mut cx));
            }

            return Ok(cx.undefined().as_value(&mut cx));
        }
    }
}

fn field_get_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let str_ptr = il2cpp_field_get_type.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.number(str_ptr as u32)
    })
}

fn method_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let str_ptr = il2cpp_method_get_name.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(str_ptr).to_str().expect("Failed to unwrap str_ptr"))
    })
}

fn type_get_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let str_ptr = il2cpp_type_get_type.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.number(str_ptr as u32)
    })
}

fn type_is_static(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let str_ptr = il2cpp_type_is_static.unwrap()(domain.value(&mut cx) as i32 as *mut c_void);
        cx.boolean(str_ptr)
    })
}

fn class_from_name(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let domain2: Handle<JsString> = cx.argument(1)?;
        let domain3: Handle<JsString> = cx.argument(2)?;
        let strPtr = il2cpp_class_from_name.unwrap()(domain.value(&mut cx) as i32 as *mut usize, CString::new(domain2.value(&mut cx)).unwrap().into_raw() as *mut c_char, CString::new(domain3.value(&mut cx)).unwrap().into_raw() as *mut c_char);
        cx.number(strPtr as u32)
    })
}

fn class_get_methods(mut cx: FunctionContext) -> JsResult<JsArray> {
    let class: Handle<JsNumber> = cx.argument(0)?;
    let class = class.value(&mut cx);
    let array = cx.empty_array();
    let mut iter: usize = 0;
    let mut i = 0;

    loop {
        let field = unsafe { il2cpp_class_get_methods.unwrap()(class as usize as *mut c_void, &mut iter) as usize };
        if field == 0 {
            break;
        }
        set!(array, &mut cx, cx.number(i as f64), cx.number(field as f64));
        i += 1;
    }

    Ok(array)
}

fn array_length(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_array_length.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
        cx.number(strPtr as u32)
    })
}

fn array_get_byte_length(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_array_get_byte_length.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
        cx.number(strPtr as u32)
    })
}

fn object_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let strPtr = il2cpp_object_get_class.unwrap()(domain.value(&mut cx) as i32 as *mut usize);
        cx.number(strPtr as u32)
    })
}

fn class_get_field_from_name(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let domain2: Handle<JsString> = cx.argument(1)?;
        let strPtr = il2cpp_class_get_field_from_name.unwrap()(domain.value(&mut cx) as i32 as *mut usize, CString::new(domain2.value(&mut cx)).unwrap().as_ptr() as *mut c_char);
        cx.number(strPtr as u32)
    })
}

fn class_get_fields(mut cx: FunctionContext) -> JsResult<JsArray> {
    let class: Handle<JsNumber> = cx.argument(0)?;
    let class = class.value(&mut cx);
    let array = cx.empty_array();
    let mut iter: usize = 0;
    let mut i = 0;

    loop {
        let field = unsafe { il2cpp_class_get_fields.unwrap()(class as usize as *mut c_void, &mut iter) as usize };
        if field == 0 {
            break;
        }
        set!(array, &mut cx, cx.number(i as f64), cx.number(field as f64));
        i += 1;
    }

    Ok(array)
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
    get_proc!(module, il2cpp_class_from_name);
    get_proc!(module, il2cpp_class_get_fields);
    get_proc!(module, il2cpp_field_get_parent);
    get_proc!(module, il2cpp_field_get_name);
    get_proc!(module, il2cpp_field_static_get_value);
    get_proc!(module, il2cpp_field_get_type);
    get_proc!(module, il2cpp_type_get_name);
    get_proc!(module, il2cpp_type_get_type);
    get_proc!(module, il2cpp_type_is_static);
    get_proc!(module, il2cpp_array_length);
    get_proc!(module, il2cpp_array_get_byte_length);
    get_proc!(module, il2cpp_object_get_class);
    get_proc!(module, il2cpp_class_get_field_from_name);
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
    set!(global_obj, cx, cx.string("il2cpp_class_from_name"), JsFunction::new(cx, class_from_name).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_class_get_fields"), JsFunction::new(cx, class_get_fields).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_field_get_parent"), JsFunction::new(cx, field_get_parent).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_field_get_name"), JsFunction::new(cx, field_get_name).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_field_static_get_value"), JsFunction::new(cx, field_static_get_value).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_field_get_type"), JsFunction::new(cx, field_get_type).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_type_get_name"), JsFunction::new(cx, type_get_name).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_type_get_type"), JsFunction::new(cx, type_get_type).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_type_is_static"), JsFunction::new(cx, type_is_static).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_array_length"), JsFunction::new(cx, array_length).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_array_get_byte_length"), JsFunction::new(cx, array_length).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_object_get_class"), JsFunction::new(cx, object_get_class).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_class_get_field_from_name"), JsFunction::new(cx, class_get_field_from_name).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("il2cpp_alloc"), JsFunction::new(cx, alloc).expect("failed to create a js_function"));
    set!(global_obj, cx, cx.string("snorestop_create_buffer"), JsFunction::new(cx, create_buffer_js).expect("failed to create a js_function"));
    set!(cx.global(), cx, "__IL2CPP", global_obj);
}
