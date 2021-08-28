#![allow(non_upper_case_globals)]

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::os::raw::{c_char, c_void};
use nodejs::neon::context::{Context, FunctionContext};
use nodejs::neon::handle::Handle;
use nodejs::neon::object::Object;
use nodejs::neon::result::{JsResult, NeonResult};
use nodejs::neon::types::{JsArray, JsArrayBuffer, JsBoolean, JsFunction, JsNumber, JsString, JsUndefined, JsValue, Value};
use paste::paste;
use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::GetProcAddress;
use crate::set;

macro_rules! gen_statics {
    ($($name: ident = ($($params: ty),*) -> $ret: ty),*) => {
        paste! {
            $(
                pub static mut $name: Option<fn ($($params),*) -> $ret> = None;
            )*
        }
    };
}

macro_rules! get_proc {
    ($module: expr, $proc_name: ident) => {
        unsafe { $proc_name = Some(transmute(GetProcAddress($module, CString::new(stringify!($proc_name)).unwrap().into_raw()))) }
    };
}

// TODO: implement a BigInt macro
// macro_rules! bigint {
//     ($cx: expr, $value: literal) => {
//         {
//             let bigint = $cx.string("BigInt");
//             let bigint: Handle<JsFunction> = $cx.global().get(*$cx, bigint)?.downcast_or_throw($cx)?;
//             let value = $cx.string(stringify!($value));
//             let output = bigint.call($cx, $cx.undefined(), [value])?;
//             output
//         }
//     }
// }

gen_statics! {
    //domain
    il2cpp_domain_get = () -> *mut c_void,
    il2cpp_domain_get_assemblies = (*mut c_void, *mut usize) -> *mut *mut c_void,
    //image
    il2cpp_image_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_image_get_filename = (*mut c_void) -> *mut c_char,
    il2cpp_image_get_assembly = (*mut c_void) -> *mut c_void,
    il2cpp_image_get_class_count = (*mut c_void) -> usize,
    il2cpp_image_get_class = (*mut c_void, usize) -> *mut usize,
    //assembly
    il2cpp_assembly_get_image = (*mut c_void) -> *mut c_void,
    //class
    il2cpp_class_from_name = (*mut usize, *mut c_char, *mut c_char) -> *mut usize,
    il2cpp_class_get_namespace = (*mut c_void) -> *mut c_char,
    il2cpp_class_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_class_get_type = (*mut c_void) -> *mut c_void,
    il2cpp_class_get_fields = (*mut c_void, *mut usize) -> *mut c_void,
    il2cpp_class_get_field_from_name = (*mut usize, *mut c_char) -> *mut *mut c_void,
    il2cpp_class_get_methods = (*mut c_void, *mut usize) -> *mut c_void,
    il2cpp_class_get_method_from_name = (*mut usize, *mut c_char, i32) -> *mut c_void,
    //field
    il2cpp_field_get_parent = (*mut c_void) -> *mut c_void,
    il2cpp_field_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_field_get_type = (*mut c_void) -> *mut c_void,
    il2cpp_field_get_value = (*mut c_void, *mut c_void, *mut u32) -> c_void,
    il2cpp_field_set_value = (*mut c_void, *mut c_void, *mut c_void) -> c_void,
    il2cpp_field_static_get_value = (*mut c_void, *mut u32) -> c_void,
    il2cpp_field_static_set_value = (*mut c_void, *mut c_void) -> c_void,
    //method
    il2cpp_method_get_class = (*mut c_void) -> *mut c_void,
    il2cpp_method_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_method_get_flags = (*mut c_void, *mut u8) -> usize,
    //type
    il2cpp_type_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_type_get_type = (*mut c_void) -> usize,
    il2cpp_type_is_static = (*mut c_void) -> bool,
    //array
    il2cpp_array_length = (*mut usize) -> usize,
    il2cpp_array_get_byte_length = (*mut usize) -> usize,
    //gc
    il2cpp_gc_disable = () -> (),
    //string
    il2cpp_string_new = (*mut c_char) -> *mut c_void,
    //runtime
    il2cpp_runtime_invoke_convert_args = (*mut usize, *mut c_void, *mut *mut c_void, u32, *mut *mut c_void) -> *mut c_void,
    il2cpp_runtime_invoke = (*mut usize, *mut usize, *mut *mut usize, u32, *mut *mut usize) -> *mut c_void,
    //object
    il2cpp_object_get_class = (*mut usize) -> *mut c_char,
    il2cpp_object_new = (*mut usize) -> *mut usize,
    //memory
    il2cpp_alloc = (usize) -> *mut c_void,
    il2cpp_free = (*mut c_void) -> ()
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
        let assembly: Handle<JsNumber> = cx.argument(0)?;
        let image = il2cpp_assembly_get_image.unwrap()(assembly.value(&mut cx) as i32 as *mut c_void);
        cx.number(image as u32)
    })
}

fn image_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let image: Handle<JsNumber> = cx.argument(0)?;
        let name = il2cpp_image_get_name.unwrap()(image.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(name).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn image_get_filename(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let image: Handle<JsNumber> = cx.argument(0)?;
        let filename = il2cpp_image_get_filename.unwrap()(image.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(filename).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn class_get_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let t = il2cpp_class_get_type.unwrap()(class.value(&mut cx) as i32 as *mut c_void);
        cx.number(t as u32)
    })
}

fn image_get_assembly(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let image: Handle<JsNumber> = cx.argument(0)?;
        let assembly = il2cpp_image_get_assembly.unwrap()(image.value(&mut cx) as i32 as *mut c_void);
        cx.number(assembly as u32)
    })
}

fn image_get_class_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let image: Handle<JsNumber> = cx.argument(0)?;
        let class_count = il2cpp_image_get_class_count.unwrap()(image.value(&mut cx) as i32 as *mut c_void);
        cx.number(class_count as u32)
    })
}

fn image_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let image: Handle<JsNumber> = cx.argument(0)?;
        let index: Handle<JsNumber> = cx.argument(1)?;
        let class = il2cpp_image_get_class.unwrap()(image.value(&mut cx) as i32 as *mut c_void, index.value(&mut cx) as usize);
        cx.number(class as u32)
    })
}

fn class_get_namespace(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let namespace = il2cpp_class_get_namespace.unwrap()(class.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(namespace).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn class_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let name = il2cpp_class_get_name.unwrap()(class.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(name).to_str().expect("Failed to unwrap strPtr"))
    })
}

fn field_get_parent(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let field: Handle<JsNumber> = cx.argument(0)?;
        let parent = il2cpp_field_get_parent.unwrap()(field.value(&mut cx) as i32 as *mut c_void);
        cx.number(parent as u32)
    })
}

fn field_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let field: Handle<JsNumber> = cx.argument(0)?;
        let name = il2cpp_field_get_name.unwrap()(field.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(name).to_str().expect("Failed to unwrap str_ptr"))
    })
}

fn field_get_value(mut cx: FunctionContext) -> JsResult<JsValue> {
    unsafe {
        let object: Handle<JsNumber> = cx.argument(0)?;
        let field: Handle<JsNumber> = cx.argument(1)?;
        let a_buffer: Handle<JsBoolean> = cx.argument(2)?;
        let d_size: Handle<JsNumber> = cx.argument(3)?;

        if a_buffer.value(&mut cx) {
            let mut res_slice = vec![0u8; d_size.value(&mut cx) as usize];
            il2cpp_field_get_value.unwrap()(object.value(&mut cx) as i32 as *mut c_void, field.value(&mut cx) as i32 as *mut c_void, res_slice.as_mut_ptr() as *mut u32);
            Ok(JsArrayBuffer::external(&mut cx, res_slice).as_value(&mut cx))
        } else if d_size.value(&mut cx) > 4.0 || d_size.value(&mut cx) == 3.0 {
            panic!("d_size cannot be greater than 4 or 3 when as_buffer is false");
        } else if d_size.value(&mut cx) == 4.0 {
            let mut res = 0;
            il2cpp_field_get_value.unwrap()(object.value(&mut cx) as i32 as *mut c_void, field.value(&mut cx) as i32 as *mut c_void, &mut res);
            Ok(cx.number(res).as_value(&mut cx))
        } else if d_size.value(&mut cx) == 2.0 {
            let res: i16 = 0;
            il2cpp_field_get_value.unwrap()(object.value(&mut cx) as i32 as *mut c_void, field.value(&mut cx) as i32 as *mut c_void, &mut (res as u32));
            Ok(cx.number(res).as_value(&mut cx))
        } else if d_size.value(&mut cx) == 1.0 {
            let res: i8 = 0;
            il2cpp_field_get_value.unwrap()(object.value(&mut cx) as i32 as *mut c_void, field.value(&mut cx) as i32 as *mut c_void, &mut (res as u32));
            Ok(cx.number(res).as_value(&mut cx))
        } else {
            Ok(cx.undefined().as_value(&mut cx))
        }
    }
}

fn field_static_get_value(mut cx: FunctionContext) -> JsResult<JsValue> {
    unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let a_buffer: Handle<JsBoolean> = cx.argument(1)?;
        let d_size: Handle<JsNumber> = cx.argument(2)?;

        if a_buffer.value(&mut cx) {
            let mut res_slice = vec![0u8; d_size.value(&mut cx) as usize];
            il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, res_slice.as_mut_ptr() as *mut u32);
            Ok(JsArrayBuffer::external(&mut cx, res_slice).as_value(&mut cx))
        } else if d_size.value(&mut cx) > 4.0 || d_size.value(&mut cx) == 3.0 {
            panic!("d_size cannot be greater than 4 or 3 when as_buffer is false");
        } else if d_size.value(&mut cx) == 4.0 {
            let mut res = 0;
            il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, &mut res);
            return Ok(cx.number(res).as_value(&mut cx));
        } else if d_size.value(&mut cx) == 2.0 {
            let res: i16 = 0;
            il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, &mut (res as u32));
            return Ok(cx.number(res).as_value(&mut cx));
        } else if d_size.value(&mut cx) == 1.0 {
            let res: i8 = 0;
            il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, &mut (res as u32));
            return Ok(cx.number(res).as_value(&mut cx));
        } else {
            Ok(cx.undefined().as_value(&mut cx))
        }
    }
}

fn field_static_set_value(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let field: Handle<JsNumber> = cx.argument(0)?;
    let field = field.value(&mut cx) as usize as *mut c_void;
    let address: Handle<JsArrayBuffer> = cx.argument(1)?;
    let ptr = cx.string("ptr");
    let address: Handle<JsNumber> = address.get(&mut cx, ptr)?.downcast_or_throw(&mut cx)?;
    let address = address.value(&mut cx) as usize as *mut c_void;
    unsafe { il2cpp_field_static_set_value.unwrap()(field, address) };
    Ok(cx.undefined())
}

fn field_set_value(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let object: Handle<JsNumber> = cx.argument(0)?;
    let object = object.value(&mut cx) as usize as *mut c_void;
    let field: Handle<JsNumber> = cx.argument(1)?;
    let field = field.value(&mut cx) as usize as *mut c_void;
    let address: Handle<JsArrayBuffer> = cx.argument(2)?;
    let ptr = cx.string("ptr");
    let address: Handle<JsNumber> = address.get(&mut cx, ptr)?.downcast_or_throw(&mut cx)?;
    let address = address.value(&mut cx) as usize as *mut c_void;
    unsafe { il2cpp_field_set_value.unwrap()(object, field, address) };
    Ok(cx.undefined())
}

fn field_get_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let field: Handle<JsNumber> = cx.argument(0)?;
        let t = il2cpp_field_get_type.unwrap()(field.value(&mut cx) as i32 as *mut c_void);
        cx.number(t as u32)
    })
}

fn method_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let class = il2cpp_method_get_class.unwrap()(method.value(&mut cx) as i32 as *mut c_void);
        return Ok(cx.number(class as u32));
    }
}

fn method_get_flags(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let flags = il2cpp_method_get_flags.unwrap()(method.value(&mut cx) as i32 as *mut c_void, 0 as *mut u8);
        cx.number(flags as u32)
    })
}

fn method_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let name = il2cpp_method_get_name.unwrap()(method.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(name).to_str().expect("Failed to unwrap str_ptr"))
    })
}

fn type_get_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let t: Handle<JsNumber> = cx.argument(0)?;
        let name = il2cpp_type_get_name.unwrap()(t.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(name).to_str().expect("Failed to unwrap str_ptr"))
    })
}

fn type_get_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let t: Handle<JsNumber> = cx.argument(0)?;
        let t = il2cpp_type_get_type.unwrap()(t.value(&mut cx) as i32 as *mut c_void);
        // TODO use bigint
        cx.number(t as u32)
    })
}

fn type_is_static(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(unsafe {
        let t: Handle<JsNumber> = cx.argument(0)?;
        let is_static = il2cpp_type_is_static.unwrap()(t.value(&mut cx) as i32 as *mut c_void);
        cx.boolean(is_static)
    })
}

fn class_from_name(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let image: Handle<JsNumber> = cx.argument(0)?;
        let namespace: Handle<JsString> = cx.argument(1)?;
        let name: Handle<JsString> = cx.argument(2)?;
        let class = il2cpp_class_from_name.unwrap()(image.value(&mut cx) as i32 as *mut usize, CString::new(namespace.value(&mut cx)).unwrap().into_raw() as *mut c_char, CString::new(name.value(&mut cx)).unwrap().into_raw() as *mut c_char);
        cx.number(class as u32)
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
        let array: Handle<JsNumber> = cx.argument(0)?;
        let length = il2cpp_array_length.unwrap()(array.value(&mut cx) as i32 as *mut usize);
        cx.number(length as u32)
    })
}

fn array_get_byte_length(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let array: Handle<JsNumber> = cx.argument(0)?;
        let length = il2cpp_array_get_byte_length.unwrap()(array.value(&mut cx) as i32 as *mut usize);
        cx.number(length as u32)
    })
}

fn object_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let object: Handle<JsNumber> = cx.argument(0)?;
        let class = il2cpp_object_get_class.unwrap()(object.value(&mut cx) as i32 as *mut usize);
        cx.number(class as u32)
    })
}

fn object_new(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let object = il2cpp_array_get_byte_length.unwrap()(class.value(&mut cx) as i32 as *mut usize);
        cx.number(object as u32)
    })
}

fn class_get_field_from_name(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let name: Handle<JsString> = cx.argument(1)?;
        let field = il2cpp_class_get_field_from_name.unwrap()(class.value(&mut cx) as i32 as *mut usize, CString::new(name.value(&mut cx)).unwrap().into_raw() as *mut c_char);
        cx.number(field as u32)
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

fn runtime_invoke_convert_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let method_info_handle: Handle<JsNumber> = cx.argument(0)?;
        let method_info = method_info_handle.value(&mut cx);
        let this_arg_handle: Handle<JsNumber> = cx.argument(1)?;
        let this_arg = this_arg_handle.value(&mut cx);
        let parameters_handle: Handle<JsArray> = cx.argument(2)?;

        let mut parameter_arr = vec![0u32; parameters_handle.len(&mut cx) as usize];

        for i in 0..parameters_handle.len(&mut cx) {
            let data: Handle<JsNumber> = parameters_handle.get(&mut cx, i).expect("Failed to get index").downcast(&mut cx).expect("Failed to downcast");
            parameter_arr[i as usize] = data.value(&mut cx) as u32;
        }

        let void_ptr = 0;

        let ptr = il2cpp_runtime_invoke_convert_args.unwrap()(
            method_info as usize as *mut usize,
            this_arg as usize as *mut c_void,
            parameter_arr.as_mut_ptr() as *mut *mut c_void,
            parameter_arr.len() as u32,
            void_ptr as usize as *mut *mut c_void,
        );

        cx.number(ptr as u32)
    })
}

fn class_get_method_from_name(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let domain2: Handle<JsString> = cx.argument(1)?;
        let domain3: Handle<JsNumber> = cx.argument(2)?;
        let str_ptr = il2cpp_class_get_method_from_name.unwrap()(domain.value(&mut cx) as i32 as *mut usize, CString::new(domain2.value(&mut cx)).unwrap().into_raw() as *mut c_char, domain3.value(&mut cx) as i32);
        cx.number(str_ptr as u32)
    })
}

fn string_new(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let domain: Handle<JsString> = cx.argument(0)?;
        let str_ptr = il2cpp_string_new.unwrap()(CString::new(domain.value(&mut cx)).unwrap().into_raw() as *mut c_char);
        cx.number(str_ptr as u32)
    })
}

fn alloc(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let size: Handle<JsNumber> = cx.argument(0)?;
    let size = size.value(&mut cx);
    if size > u32::MAX as f64 {
        panic!("Cannot allocate >4GB buffers!");
    }
    let buffer = unsafe { il2cpp_alloc.unwrap()(size as i32 as usize) };
    Ok(cx.number(buffer as usize as f64))
}

fn gc_disable(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // unsafe {
    //     il2cpp_gc_disable;
    // }

    Ok(cx.undefined())
}

pub(crate) fn load_functions<'a, C: Context<'a>>(module: HMODULE, cx: &mut C) -> NeonResult<()> {
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
    get_proc!(module, il2cpp_field_get_value);
    get_proc!(module, il2cpp_field_static_set_value);
    get_proc!(module, il2cpp_field_set_value);
    get_proc!(module, il2cpp_field_get_type);
    get_proc!(module, il2cpp_type_get_name);
    get_proc!(module, il2cpp_type_get_type);
    get_proc!(module, il2cpp_type_is_static);
    get_proc!(module, il2cpp_array_length);
    get_proc!(module, il2cpp_array_get_byte_length);
    get_proc!(module, il2cpp_object_get_class);
    get_proc!(module, il2cpp_object_new);
    get_proc!(module, il2cpp_class_get_field_from_name);
    get_proc!(module, il2cpp_class_get_methods);
    get_proc!(module, il2cpp_method_get_flags);
    get_proc!(module, il2cpp_method_get_name);
    get_proc!(module, il2cpp_class_get_method_from_name);
    get_proc!(module, il2cpp_runtime_invoke_convert_args);
    get_proc!(module, il2cpp_runtime_invoke);
    get_proc!(module, il2cpp_class_get_type);
    get_proc!(module, il2cpp_string_new);
    get_proc!(module, il2cpp_gc_disable);
    get_proc!(module, il2cpp_alloc);

    {
    }

    // TODO: make all of these functions actually 64-bit pointer size safe (bigint?)
    //domain
    set!(global_obj, cx, cx.string("il2cpp_domain_get"), JsFunction::new(cx, domain_get)?);
    set!(global_obj, cx, cx.string("il2cpp_domain_get_assemblies"), JsFunction::new(cx, domain_get_assemblies)?);
    //image
    set!(global_obj, cx, cx.string("il2cpp_image_get_name"), JsFunction::new(cx, image_get_name)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_filename"), JsFunction::new(cx, image_get_filename)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_assembly"), JsFunction::new(cx, image_get_assembly)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_class_count"), JsFunction::new(cx, image_get_class_count)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_class"), JsFunction::new(cx, image_get_class)?);
    //assembly
    set!(global_obj, cx, cx.string("il2cpp_assembly_get_image"), JsFunction::new(cx, assembly_get_image)?);
    //class
    set!(global_obj, cx, cx.string("il2cpp_class_from_name"), JsFunction::new(cx, class_from_name)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_namespace"), JsFunction::new(cx, class_get_namespace)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_name"), JsFunction::new(cx, class_get_name)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_type"), JsFunction::new(cx, class_get_type)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_fields"), JsFunction::new(cx, class_get_fields)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_field_from_name"), JsFunction::new(cx, class_get_field_from_name)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_methods"), JsFunction::new(cx, class_get_methods)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_method_from_name"), JsFunction::new(cx, class_get_method_from_name)?);
    //field
    set!(global_obj, cx, cx.string("il2cpp_field_get_parent"), JsFunction::new(cx, field_get_parent)?);
    set!(global_obj, cx, cx.string("il2cpp_field_get_name"), JsFunction::new(cx, field_get_name)?);
    set!(global_obj, cx, cx.string("il2cpp_field_get_type"), JsFunction::new(cx, field_get_type)?);
    set!(global_obj, cx, cx.string("il2cpp_field_get_value"), JsFunction::new(cx, field_get_value)?);
    set!(global_obj, cx, cx.string("il2cpp_field_set_value"), JsFunction::new(cx, field_set_value)?);
    set!(global_obj, cx, cx.string("il2cpp_field_static_get_value"), JsFunction::new(cx, field_static_get_value)?);
    set!(global_obj, cx, cx.string("il2cpp_field_static_set_value"), JsFunction::new(cx, field_static_set_value)?);
    //method
    set!(global_obj, cx, cx.string("il2cpp_method_get_name"), JsFunction::new(cx, method_get_name)?);
    set!(global_obj, cx, cx.string("il2cpp_method_get_flags"), JsFunction::new(cx, method_get_flags)?);
    //type
    set!(global_obj, cx, cx.string("il2cpp_type_get_name"), JsFunction::new(cx, type_get_name)?);
    set!(global_obj, cx, cx.string("il2cpp_type_get_type"), JsFunction::new(cx, type_get_type)?);
    set!(global_obj, cx, cx.string("il2cpp_type_is_static"), JsFunction::new(cx, type_is_static)?);
    //array
    set!(global_obj, cx, cx.string("il2cpp_array_length"), JsFunction::new(cx, array_length)?);
    set!(global_obj, cx, cx.string("il2cpp_array_get_byte_length"), JsFunction::new(cx, array_length)?);
    //gc
    set!(global_obj, cx, cx.string("il2cpp_gc_disable"), JsFunction::new(cx, gc_disable)?);
    //string
    set!(global_obj, cx, cx.string("il2cpp_string_new"), JsFunction::new(cx, string_new)?);
    //runtime
    set!(global_obj, cx, cx.string("il2cpp_runtime_invoke_convert_args"), JsFunction::new(cx, runtime_invoke_convert_args)?);
    // set!(global_obj, cx, cx.string("il2cpp_runtime_invoke"), JsFunction::new(cx, runtime_invoke).expect("failed to create a js_function"));
    //object
    set!(global_obj, cx, cx.string("il2cpp_object_get_class"), JsFunction::new(cx, object_get_class)?);
    set!(global_obj, cx, cx.string("il2cpp_object_new"), JsFunction::new(cx, object_new)?);
    //memory
    set!(global_obj, cx, cx.string("il2cpp_alloc"), JsFunction::new(cx, alloc)?);
    set!(cx.global(), cx, "__IL2CPP", global_obj);
    Ok(())
}
