#![allow(non_upper_case_globals)]

use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::os::raw::{c_char, c_void};
use nodejs::neon::context::{Context, FunctionContext};
use nodejs::neon::handle::Handle;
use nodejs::neon::object::Object;
use nodejs::neon::result::{JsResult, NeonResult};
use nodejs::neon::types::{JsArray, JsArrayBuffer, JsBoolean, JsFunction, JsNumber, JsObject, JsString, JsUndefined, JsValue, Value};
use paste::paste;
use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::GetProcAddress;
use crate::gc::create_gc_handle;
use crate::memview::View;
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
    il2cpp_domain_assembly_open = (*mut c_void, *mut c_char) -> *mut c_void,
    //image
    il2cpp_image_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_image_get_filename = (*mut c_void) -> *mut c_char,
    il2cpp_image_get_assembly = (*mut c_void) -> *mut c_void,
    il2cpp_image_get_class_count = (*mut c_void) -> usize,
    il2cpp_image_get_class = (*mut c_void, usize) -> *mut usize,
    il2cpp_image_get_entry_point = (*mut c_void) -> *mut c_void,
    //assembly
    il2cpp_assembly_get_image = (*mut c_void) -> *mut c_void,
    //class
    il2cpp_class_from_name = (*mut usize, *mut c_char, *mut c_char) -> *mut usize,
    il2cpp_class_from_type = (*mut c_void) -> *mut c_void,
    il2cpp_class_get_namespace = (*mut c_void) -> *mut c_char,
    il2cpp_class_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_class_get_type = (*mut c_void) -> *mut c_void,
    il2cpp_class_get_fields = (*mut c_void, *mut usize) -> *mut c_void,
    il2cpp_class_get_field_from_name = (*mut usize, *mut c_char) -> *mut *mut c_void,
    il2cpp_class_get_methods = (*mut c_void, *mut usize) -> *mut c_void,
    il2cpp_class_get_method_from_name = (*mut usize, *mut c_char, i32) -> *mut c_void,
    il2cpp_class_value_size = (*mut c_void, *mut u32) -> *mut c_void,
    il2cpp_class_is_enum = (*mut c_void) -> bool,
    il2cpp_class_enum_basetype = (*mut c_void) -> *mut c_void,
    il2cpp_class_from_il2cpp_type = (*mut c_void) -> *mut c_void,
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
    il2cpp_method_get_param_count = (*mut c_void) -> usize,
    il2cpp_method_get_param_name = (*mut c_void, u32) -> usize,
    il2cpp_method_get_param = (*mut c_void, u32) -> usize,
    il2cpp_method_get_return_type = (*mut c_void) -> usize,
    il2cpp_method_is_instance = (*mut c_void) -> bool,
    il2cpp_method_get_declaring_type = (*mut c_void) -> *mut c_void,
    //type
    il2cpp_type_get_name = (*mut c_void) -> *mut c_char,
    il2cpp_type_get_type = (*mut c_void) -> usize,
    il2cpp_type_is_static = (*mut c_void) -> bool,
    il2cpp_type_is_byref = (*mut c_void) -> bool,
    il2cpp_type_is_pointer_type = (*mut c_void) -> bool,
    il2cpp_type_get_assembly_qualified_name = (*mut c_void) -> *mut c_char,
    il2cpp_type_get_class_or_element_class = (*mut c_void) -> *mut c_void,
    il2cpp_type_get_object = (*mut c_void) -> *mut c_void,
    il2cpp_type_get_attrs = (*mut c_void) -> *mut c_void,
    //array
    il2cpp_array_length = (*mut usize) -> usize,
    il2cpp_array_get_byte_length = (*mut usize) -> usize,
    il2cpp_array_object_header_size = () -> u32,
    il2cpp_array_element_size = (*mut usize) -> usize,
    il2cpp_array_class_get = (*mut usize, u32) -> *mut c_void,
    //gc
    il2cpp_gc_disable = () -> (),
    il2cpp_gc_enable = () -> (),
    //gchandle
    il2cpp_gchandle_new = (*mut c_void, bool) -> u32,
    il2cpp_gchandle_new_weakref = (*mut c_void, bool) -> u32,
    il2cpp_gchandle_get_target = (u32) -> *mut c_void,
    //string
    il2cpp_string_new = (*mut c_char) -> *mut c_void,
    //runtime
    il2cpp_runtime_invoke_convert_args = (*mut usize, *mut c_void, *mut *mut c_void, u32, *mut *mut c_void) -> *mut c_void,
    il2cpp_runtime_invoke = (*mut usize, *mut usize, *mut *mut usize, *mut usize) -> *mut c_void,
    //object
    il2cpp_object_get_class = (*mut usize) -> *mut c_char,
    il2cpp_object_header_size = () -> u32,
    il2cpp_object_get_size = (*mut usize) -> u32,
    il2cpp_object_new = (*mut usize) -> *mut c_void,
    //memory
    il2cpp_alloc = (usize) -> *mut c_void,
    il2cpp_free = (*mut c_void) -> (),
    il2cpp_thread_attach = (*mut c_void) -> (),
    //value
    il2cpp_value_box = (*mut c_void, *mut c_void) -> *mut c_void
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

fn domain_assembly_open(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let domain: Handle<JsNumber> = cx.argument(0)?;
    let assembly: Handle<JsString> = cx.argument(1)?;
    let assemblyRes = unsafe { il2cpp_domain_assembly_open.unwrap()(domain.value(&mut cx) as i32 as *mut c_void, CString::new(assembly.value(&mut cx)).unwrap().into_raw() as *mut c_char) };    

    Ok(cx.number(assemblyRes as u32))
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

fn image_get_entry_point(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let image: Handle<JsNumber> = cx.argument(0)?;
        let class_count = il2cpp_image_get_entry_point.unwrap()(image.value(&mut cx) as i32 as *mut c_void);
        cx.number(class_count as u32)
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

fn class_from_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let ptr = il2cpp_class_from_type.unwrap()(class.value(&mut cx) as i32 as *mut c_void);
        cx.number(ptr as usize as f64)
    })
}

fn class_value_size(mut cx: FunctionContext) -> JsResult<JsArray> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let arr = cx.empty_array();
        let mut align: u32 = 0;
        let ptr = il2cpp_class_value_size.unwrap()(class.value(&mut cx) as i32 as *mut c_void, &mut align);
        set!(arr, &mut cx, cx.number(0), cx.number(ptr as usize as f64));
        set!(arr, &mut cx, cx.number(1), cx.number(align as usize as f64));
        arr
    })
}

fn class_is_enum(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let bool = il2cpp_class_is_enum.unwrap()(class.value(&mut cx) as i32 as *mut c_void);
        cx.boolean(bool)
    })
}

fn class_enum_basetype(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let _type = il2cpp_class_enum_basetype.unwrap()(class.value(&mut cx) as i32 as *mut c_void);
        cx.number(_type as usize as f64)
    })
}

fn class_from_il2cpp_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        let _type = il2cpp_class_from_il2cpp_type.unwrap()(class.value(&mut cx) as i32 as *mut c_void);
        cx.number(_type as usize as f64)
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

fn field_get_value(mut cx: FunctionContext) -> JsResult<JsObject> {
    unsafe {
        let object: Handle<JsNumber> = cx.argument(0)?;
        let domain: Handle<JsNumber> = cx.argument(1)?;
        let allocSize: Handle<JsNumber> = cx.argument(2)?;
        let mut res_slice = Box::new(vec![0u8; allocSize.value(&mut cx) as usize]);
        il2cpp_field_get_value.unwrap()(object.value(&mut cx) as u32 as *mut c_void, domain.value(&mut cx) as u32 as *mut c_void, res_slice.as_mut_ptr() as *mut u32);
        let rustIsKindaBadLlol = res_slice.as_mut_ptr() as usize;
        //Todo: Deallocate
        Box::into_raw(res_slice);
        View::new(cx, rustIsKindaBadLlol)
    }
}

fn field_static_get_value(mut cx: FunctionContext) -> JsResult<JsObject> {
    unsafe {
        let domain: Handle<JsNumber> = cx.argument(0)?;
        let allocSize: Handle<JsNumber> = cx.argument(1)?;
        let mut res_slice = Box::new(vec![0u8; allocSize.value(&mut cx) as usize]);
        il2cpp_field_static_get_value.unwrap()(domain.value(&mut cx) as u32 as *mut c_void, res_slice.as_mut_ptr() as *mut u32);
        let rustIsKindaBadLlol = res_slice.as_mut_ptr() as usize;
        //Todo: Deallocate
        Box::into_raw(res_slice);
        View::new(cx, rustIsKindaBadLlol)
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
        let t = il2cpp_field_get_type.unwrap()(field.value(&mut cx) as u32 as *mut c_void);
        cx.number(t as u32)
    })
}

fn method_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let class = il2cpp_method_get_class.unwrap()(method.value(&mut cx) as u32 as *mut c_void);
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

fn method_get_param_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let param_count = il2cpp_method_get_param_count.unwrap()(method.value(&mut cx) as i32 as *mut c_void);
        cx.number(param_count as u32)
    })
}

fn method_get_param(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let paramidx: Handle<JsNumber> = cx.argument(1)?;
        let param_type = il2cpp_method_get_param.unwrap()(method.value(&mut cx) as i32 as *mut c_void, paramidx.value(&mut cx) as u32);
        cx.number(param_type as u32)
    })
}

fn method_get_param_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let paramidx: Handle<JsNumber> = cx.argument(1)?;
        let param_type = il2cpp_method_get_param_name.unwrap()(method.value(&mut cx) as i32 as *mut c_void, paramidx.value(&mut cx) as u32);
        cx.string(CStr::from_ptr(param_type as *const i8).to_str().expect("Failed to unwrap str_ptr"))
    })
}

fn method_get_return_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let param_type = il2cpp_method_get_return_type.unwrap()(method.value(&mut cx) as usize as *mut c_void);
        cx.number(param_type as f64)
    })
}

fn method_is_instance(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let param_type = il2cpp_method_is_instance.unwrap()(method.value(&mut cx) as usize as *mut c_void);
        cx.boolean(param_type)
    })
}

fn method_get_declaring_type(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let method: Handle<JsNumber> = cx.argument(0)?;
        let param_type: *mut c_void = il2cpp_method_get_declaring_type.unwrap()(method.value(&mut cx) as usize as *mut c_void); 
        cx.number(param_type as usize as f64)
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

fn type_is_byref(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(unsafe {
        let t: Handle<JsNumber> = cx.argument(0)?;
        let is_static = il2cpp_type_is_byref.unwrap()(t.value(&mut cx) as i32 as *mut c_void);
        cx.boolean(is_static)
    })
}

fn type_is_pointer_type(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(unsafe {
        let t: Handle<JsNumber> = cx.argument(0)?;
        let is_static = il2cpp_type_is_pointer_type.unwrap()(t.value(&mut cx) as i32 as *mut c_void);
        cx.boolean(is_static)
    })
}

fn type_get_assembly_qualified_name(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(unsafe {
        let field: Handle<JsNumber> = cx.argument(0)?;
        let name = il2cpp_type_get_assembly_qualified_name.unwrap()(field.value(&mut cx) as i32 as *mut c_void);
        cx.string(CStr::from_ptr(name).to_str().expect("Failed to unwrap str_ptr"))
    })
}

fn type_get_class_or_element_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let field: Handle<JsNumber> = cx.argument(0)?;
        let ptr = il2cpp_type_get_class_or_element_class.unwrap()(field.value(&mut cx) as i32 as *mut c_void);
        cx.number(ptr as u32)
    })
}

fn type_get_object(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let field: Handle<JsNumber> = cx.argument(0)?;
        il2cpp_gc_disable.unwrap()();
        let str_ptr = il2cpp_type_get_object.unwrap()(field.value(&mut cx) as i32 as *mut c_void);
        let str_handle = il2cpp_gchandle_new.unwrap()(str_ptr, false);
        il2cpp_gc_enable.unwrap()();
        cx.number(str_handle as u32)
    })
}

fn type_get_attrs(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let field: Handle<JsNumber> = cx.argument(0)?;
        let ptr = il2cpp_type_get_attrs.unwrap()(field.value(&mut cx) as i32 as *mut c_void);
        cx.number(ptr as u32)
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

fn array_object_header_size(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe { cx.number(il2cpp_array_object_header_size.unwrap()()) })
}

fn array_element_size(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let array: Handle<JsNumber> = cx.argument(0)?;
        let length = il2cpp_array_element_size.unwrap()(array.value(&mut cx) as i32 as *mut usize);
        cx.number(length as u32)
    })
}

fn array_class_get(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let array: Handle<JsNumber> = cx.argument(0)?;
        let rank: Handle<JsNumber> = cx.argument(1)?;
        let ptr = il2cpp_array_class_get.unwrap()(array.value(&mut cx) as i32 as *mut usize, rank.value(&mut cx) as u32);
        cx.number(ptr as u32 as f64)
    })
}

fn object_get_class(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let object: Handle<JsNumber> = cx.argument(0)?;
        let class = il2cpp_object_get_class.unwrap()(object.value(&mut cx) as i32 as *mut usize);
        cx.number(class as u32)
    })
}

fn object_get_size(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let object: Handle<JsNumber> = cx.argument(0)?;
        let class = il2cpp_object_get_size.unwrap()(object.value(&mut cx) as i32 as *mut usize);
        cx.number(class as u32)
    })
}

fn object_header_size(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let class = il2cpp_object_header_size.unwrap()();
        cx.number(class as u32)
    })
}

fn object_new(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let class: Handle<JsNumber> = cx.argument(0)?;
        il2cpp_gc_disable.unwrap()();

        let object = il2cpp_object_new.unwrap()(class.value(&mut cx) as i32 as *mut usize);

        let handle = il2cpp_gchandle_new.unwrap()(object, false);

        il2cpp_gc_enable.unwrap()();
        cx.number(handle as u32)
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
        let exception_ptr_handle: Handle<JsNumber> = cx.argument(3)?;
        let exception_ptr = exception_ptr_handle.value(&mut cx);

        let mut parameter_arr = vec![0u32; parameters_handle.len(&mut cx) as usize];

        for i in 0..parameters_handle.len(&mut cx) {
            let data: Handle<JsNumber> = parameters_handle.get(&mut cx, i).expect("Failed to get index").downcast(&mut cx).expect("Failed to downcast");
            parameter_arr[i as usize] = data.value(&mut cx) as u32;
        }

        let void_ptr = &mut [0 as usize; 1];

        il2cpp_gc_disable.unwrap()();

        let paramArrPtr = parameter_arr.as_mut_ptr();

        let voidPtrPtr = void_ptr.as_mut_ptr();

        println!("PreInvoke {}, {}, {}, {}, {}", method_info, this_arg, paramArrPtr as usize, parameter_arr.len(), voidPtrPtr as usize);

        let ptr = il2cpp_runtime_invoke_convert_args.unwrap()(
            method_info as usize as *mut usize,
            this_arg as usize as *mut c_void,
            paramArrPtr as *mut *mut c_void,
            parameter_arr.len() as u32,
            exception_ptr as usize as *mut *mut c_void,
        );

        println!("PostInvoke {}", ptr as usize);

        let handle = il2cpp_gchandle_new.unwrap()(ptr, false);

        il2cpp_gc_enable.unwrap()();

        cx.number(handle as u32)
    })
}

fn runtime_invoke(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(unsafe {
        let method_info_handle: Handle<JsNumber> = cx.argument(0)?;
        let method_info = method_info_handle.value(&mut cx);
        let this_arg_handle: Handle<JsNumber> = cx.argument(1)?;
        let this_arg = this_arg_handle.value(&mut cx);
        let parameters_handle: Handle<JsArray> = cx.argument(2)?;
        let exception_ptr_handle: Handle<JsNumber> = cx.argument(3)?;
        let exception_ptr = exception_ptr_handle.value(&mut cx);

        let mut parameter_arr = vec![0u32; parameters_handle.len(&mut cx) as usize];

        for i in 0..parameters_handle.len(&mut cx) {
            let data: Handle<JsNumber> = parameters_handle.get(&mut cx, i).expect("Failed to get index").downcast(&mut cx).expect("Failed to downcast");
            parameter_arr[i as usize] = data.value(&mut cx) as u32;
        }

        il2cpp_gc_disable.unwrap()();

        let paramArrPtr = parameter_arr.as_mut_ptr();

        println!("PreInvoke {}, {}, {}, {}", method_info, this_arg, paramArrPtr as usize, exception_ptr as usize);

        let ptr = il2cpp_runtime_invoke.unwrap()(
            method_info as usize as *mut usize,
            this_arg as usize as *mut usize,
            paramArrPtr as *mut *mut usize,
            exception_ptr as usize as *mut usize,
        );

        println!("PostInvoke {}", ptr as usize);

        let handle = il2cpp_gchandle_new.unwrap()(ptr, false);

        il2cpp_gc_enable.unwrap()();

        cx.number(handle as u32)
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
        il2cpp_gc_disable.unwrap()();
        let str_ptr = il2cpp_string_new.unwrap()(CString::new(domain.value(&mut cx)).unwrap().into_raw() as *mut c_char);
        let str_handle = il2cpp_gchandle_new.unwrap()(str_ptr, false);
        il2cpp_gc_enable.unwrap()();
        cx.number(str_handle as u32)
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
    unsafe {
        il2cpp_gc_disable.unwrap()();
    }

    Ok(cx.undefined())
}

fn gchandle_get_target(mut cx: FunctionContext) -> JsResult<JsNumber> {
    unsafe {
        let handle: Handle<JsNumber> = cx.argument(0)?;
        let handle = handle.value(&mut cx);
        let pointer = il2cpp_gchandle_get_target.unwrap()(handle as u32);
        Ok(cx.number(pointer as usize as f64))
    }
}

fn gchandle_new(mut cx: FunctionContext) -> JsResult<JsNumber> {
    unsafe {
        let ptr: Handle<JsNumber> = cx.argument(0)?;
        let pinned: Handle<JsBoolean> = cx.argument(1)?;
        let handle = il2cpp_gchandle_new.unwrap()(ptr.value(&mut cx) as usize as *mut c_void, pinned.value(&mut cx));
        Ok(cx.number(handle as usize as f64))
    }
}

fn value_box(mut cx: FunctionContext) -> JsResult<JsNumber> {
    unsafe {
        let handle: Handle<JsNumber> = cx.argument(0)?;
        let handle = handle.value(&mut cx);
        let handle2: Handle<JsNumber> = cx.argument(1)?;
        let handle2 = handle2.value(&mut cx);
        il2cpp_gc_disable.unwrap()();
        let pointer = il2cpp_value_box.unwrap()(handle as i32 as *mut c_void, handle2 as i32 as *mut c_void);
        let gchandle = il2cpp_gchandle_new.unwrap()(pointer, false);
        il2cpp_gc_enable.unwrap()();
        Ok(cx.number(gchandle as u32))
    }
}

pub(crate) fn get_early_funcs(module: HMODULE) {
    get_proc!(module, il2cpp_domain_get);
    get_proc!(module, il2cpp_thread_attach);
}

pub(crate) fn load_functions<'a, C: Context<'a>>(module: HMODULE, cx: &mut C) -> NeonResult<()> {
    let global_obj = cx.empty_object();
    get_proc!(module, il2cpp_domain_get_assemblies);
    get_proc!(module, il2cpp_domain_assembly_open);
    get_proc!(module, il2cpp_assembly_get_image);
    get_proc!(module, il2cpp_image_get_name);
    get_proc!(module, il2cpp_image_get_filename);
    get_proc!(module, il2cpp_image_get_assembly);
    get_proc!(module, il2cpp_image_get_class_count);
    get_proc!(module, il2cpp_image_get_class);
    get_proc!(module, il2cpp_image_get_entry_point);
    get_proc!(module, il2cpp_class_get_namespace);
    get_proc!(module, il2cpp_class_get_name);
    get_proc!(module, il2cpp_class_from_name);
    get_proc!(module, il2cpp_class_from_type);
    get_proc!(module, il2cpp_class_get_fields);
    get_proc!(module, il2cpp_class_value_size);
    get_proc!(module, il2cpp_class_enum_basetype);
    get_proc!(module, il2cpp_class_from_il2cpp_type);
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
    get_proc!(module, il2cpp_type_is_byref);
    get_proc!(module, il2cpp_type_is_pointer_type);
    get_proc!(module, il2cpp_type_get_assembly_qualified_name);
    get_proc!(module, il2cpp_type_get_class_or_element_class);
    get_proc!(module, il2cpp_type_get_object);
    get_proc!(module, il2cpp_type_get_attrs);
    get_proc!(module, il2cpp_array_length);
    get_proc!(module, il2cpp_array_get_byte_length);
    get_proc!(module, il2cpp_array_object_header_size);
    get_proc!(module, il2cpp_array_element_size);
    get_proc!(module, il2cpp_array_class_get);
    get_proc!(module, il2cpp_object_get_class);
    get_proc!(module, il2cpp_object_header_size);
    get_proc!(module, il2cpp_object_get_size);
    get_proc!(module, il2cpp_object_new);
    get_proc!(module, il2cpp_class_get_field_from_name);
    get_proc!(module, il2cpp_class_get_methods);
    get_proc!(module, il2cpp_class_is_enum);
    get_proc!(module, il2cpp_method_get_flags);
    get_proc!(module, il2cpp_method_get_name);
    get_proc!(module, il2cpp_method_get_param_count);
    get_proc!(module, il2cpp_method_get_param_name);
    get_proc!(module, il2cpp_method_get_param);
    get_proc!(module, il2cpp_method_get_return_type);
    get_proc!(module, il2cpp_method_is_instance);
    get_proc!(module, il2cpp_method_get_declaring_type);
    get_proc!(module, il2cpp_class_get_method_from_name);
    get_proc!(module, il2cpp_runtime_invoke_convert_args);
    get_proc!(module, il2cpp_runtime_invoke);
    get_proc!(module, il2cpp_class_get_type);
    get_proc!(module, il2cpp_string_new);
    get_proc!(module, il2cpp_gc_disable);
    get_proc!(module, il2cpp_gc_enable);
    get_proc!(module, il2cpp_gchandle_new);
    get_proc!(module, il2cpp_gchandle_get_target);
    get_proc!(module, il2cpp_alloc);
    get_proc!(module, il2cpp_free);
    get_proc!(module, il2cpp_value_box);

    {
    }

    // TODO: make all of these functions actually 64-bit pointer size safe (bigint?)
    //domain
    set!(global_obj, cx, cx.string("il2cpp_domain_get"), JsFunction::new(cx, domain_get)?);
    set!(global_obj, cx, cx.string("il2cpp_domain_get_assemblies"), JsFunction::new(cx, domain_get_assemblies)?);
    set!(global_obj, cx, cx.string("il2cpp_domain_assembly_open"), JsFunction::new(cx, domain_assembly_open)?);
    //image
    set!(global_obj, cx, cx.string("il2cpp_image_get_name"), JsFunction::new(cx, image_get_name)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_filename"), JsFunction::new(cx, image_get_filename)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_assembly"), JsFunction::new(cx, image_get_assembly)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_class_count"), JsFunction::new(cx, image_get_class_count)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_class"), JsFunction::new(cx, image_get_class)?);
    set!(global_obj, cx, cx.string("il2cpp_image_get_entry_point"), JsFunction::new(cx, image_get_entry_point)?);
    //assembly
    set!(global_obj, cx, cx.string("il2cpp_assembly_get_image"), JsFunction::new(cx, assembly_get_image)?);
    //class
    set!(global_obj, cx, cx.string("il2cpp_class_from_name"), JsFunction::new(cx, class_from_name)?);
    set!(global_obj, cx, cx.string("il2cpp_class_from_type"), JsFunction::new(cx, class_from_type)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_namespace"), JsFunction::new(cx, class_get_namespace)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_name"), JsFunction::new(cx, class_get_name)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_type"), JsFunction::new(cx, class_get_type)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_fields"), JsFunction::new(cx, class_get_fields)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_field_from_name"), JsFunction::new(cx, class_get_field_from_name)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_methods"), JsFunction::new(cx, class_get_methods)?);
    set!(global_obj, cx, cx.string("il2cpp_class_get_method_from_name"), JsFunction::new(cx, class_get_method_from_name)?);
    set!(global_obj, cx, cx.string("il2cpp_class_value_size"), JsFunction::new(cx, class_value_size)?);
    set!(global_obj, cx, cx.string("il2cpp_class_is_enum"), JsFunction::new(cx, class_is_enum)?);
    set!(global_obj, cx, cx.string("il2cpp_class_enum_basetype"), JsFunction::new(cx, class_enum_basetype)?);
    set!(global_obj, cx, cx.string("il2cpp_class_from_il2cpp_type"), JsFunction::new(cx, class_from_il2cpp_type)?);
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
    set!(global_obj, cx, cx.string("il2cpp_method_get_param_count"), JsFunction::new(cx, method_get_param_count)?);
    set!(global_obj, cx, cx.string("il2cpp_method_get_param_name"), JsFunction::new(cx, method_get_param_name)?);
    set!(global_obj, cx, cx.string("il2cpp_method_get_param"), JsFunction::new(cx, method_get_param)?);
    set!(global_obj, cx, cx.string("il2cpp_method_get_return_type"), JsFunction::new(cx, method_get_return_type)?);
    set!(global_obj, cx, cx.string("il2cpp_method_is_instance"), JsFunction::new(cx, method_is_instance)?);
    set!(global_obj, cx, cx.string("il2cpp_method_get_declaring_type"), JsFunction::new(cx, method_get_declaring_type)?);
    //type
    set!(global_obj, cx, cx.string("il2cpp_type_get_name"), JsFunction::new(cx, type_get_name)?);
    set!(global_obj, cx, cx.string("il2cpp_type_get_type"), JsFunction::new(cx, type_get_type)?);
    set!(global_obj, cx, cx.string("il2cpp_type_is_static"), JsFunction::new(cx, type_is_static)?);
    set!(global_obj, cx, cx.string("il2cpp_type_is_byref"), JsFunction::new(cx, type_is_byref)?);
    set!(global_obj, cx, cx.string("il2cpp_type_is_pointer_type"), JsFunction::new(cx, type_is_pointer_type)?);
    set!(global_obj, cx, cx.string("il2cpp_type_get_assembly_qualified_name"), JsFunction::new(cx, type_get_assembly_qualified_name)?);
    set!(global_obj, cx, cx.string("il2cpp_type_get_class_or_element_class"), JsFunction::new(cx, type_get_class_or_element_class)?);
    set!(global_obj, cx, cx.string("il2cpp_type_get_object"), JsFunction::new(cx, type_get_object)?);
    set!(global_obj, cx, cx.string("il2cpp_type_get_attrs"), JsFunction::new(cx, type_get_attrs)?);
    //array
    set!(global_obj, cx, cx.string("il2cpp_array_length"), JsFunction::new(cx, array_length)?);
    set!(global_obj, cx, cx.string("il2cpp_array_get_byte_length"), JsFunction::new(cx, array_get_byte_length)?);
    set!(global_obj, cx, cx.string("il2cpp_array_object_header_size"), JsFunction::new(cx, array_object_header_size)?);
    set!(global_obj, cx, cx.string("il2cpp_array_element_size"), JsFunction::new(cx, array_element_size)?);
    set!(global_obj, cx, cx.string("il2cpp_array_class_get"), JsFunction::new(cx, array_class_get)?);
    //gc
    set!(global_obj, cx, cx.string("il2cpp_gc_disable"), JsFunction::new(cx, gc_disable)?);
    //gchandle
    set!(global_obj, cx, cx.string("il2cpp_gchandle_get_target"), JsFunction::new(cx, gchandle_get_target)?);
    set!(global_obj, cx, cx.string("il2cpp_gchandle_new"), JsFunction::new(cx, gchandle_new)?);
    //string
    set!(global_obj, cx, cx.string("il2cpp_string_new"), JsFunction::new(cx, string_new)?);
    //runtime
    set!(global_obj, cx, cx.string("il2cpp_runtime_invoke_convert_args"), JsFunction::new(cx, runtime_invoke_convert_args)?);
    set!(global_obj, cx, cx.string("il2cpp_runtime_invoke"), JsFunction::new(cx, runtime_invoke)?);
    //object
    set!(global_obj, cx, cx.string("il2cpp_object_get_class"), JsFunction::new(cx, object_get_class)?);
    set!(global_obj, cx, cx.string("il2cpp_object_header_size"), JsFunction::new(cx, object_header_size)?);
    set!(global_obj, cx, cx.string("il2cpp_object_get_size"), JsFunction::new(cx, object_get_size)?);
    set!(global_obj, cx, cx.string("il2cpp_object_new"), JsFunction::new(cx, object_new)?);
    //memory
    set!(global_obj, cx, cx.string("il2cpp_alloc"), JsFunction::new(cx, alloc)?);
    //value
    set!(global_obj, cx, cx.string("il2cpp_value_box"), JsFunction::new(cx, value_box)?);
    set!(cx.global(), cx, "__IL2CPP", global_obj);
    Ok(())
}
