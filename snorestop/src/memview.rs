use std::ffi::{CStr, c_void, CString};

use nodejs::neon::context::{Context, FunctionContext};
use nodejs::neon::handle::Handle;
use nodejs::neon::object::Object;
use nodejs::neon::prelude::Finalize;
use nodejs::neon::result::{JsResult, NeonResult};
use nodejs::neon::types::{JsBox, JsFunction, JsNumber, JsObject, JsString, JsUndefined, JsBoolean};

use crate::{set, get};
use std::sync::Mutex;
use widestring::{U16CStr, U16String, U16Str};

pub struct View {
    head: usize,
    offset: usize,
    was_il2cpp: bool
}

impl Finalize for View {
    fn finalize<'a, C: Context<'a>>(self, _cx: &mut C) {
        // TODO: Figure out how to automatically free this memory (refcount?)
    }
}

impl View {
    fn new<'a, C: Context<'a>>(mut cx: C, pointer: usize) -> JsResult<'a, JsObject> {
        let this = cx.empty_object();
        let box_var = JsBox::new(&mut cx, Mutex::new(View {
            head: pointer,
            offset: 0,
            was_il2cpp: false
        }));

        set!(this, &mut cx, cx.string("_box"), box_var);
        set!(this, &mut cx, cx.string("getHead"), JsFunction::new(&mut cx, get_head)?);
        set!(this, &mut cx, cx.string("getOffset"), JsFunction::new(&mut cx, get_offset)?);
        set!(this, &mut cx, cx.string("readU8"), JsFunction::new(&mut cx, read_u8)?);
        set!(this, &mut cx, cx.string("readI8"), JsFunction::new(&mut cx, read_i8)?);
        set!(this, &mut cx, cx.string("readU16"), JsFunction::new(&mut cx, read_u16)?);
        set!(this, &mut cx, cx.string("readI16"), JsFunction::new(&mut cx, read_i16)?);
        set!(this, &mut cx, cx.string("readU32"), JsFunction::new(&mut cx, read_u32)?);
        set!(this, &mut cx, cx.string("readI32"), JsFunction::new(&mut cx, read_i32)?);
        set!(this, &mut cx, cx.string("readPtr"), JsFunction::new(&mut cx, read_ptr)?);
        set!(this, &mut cx, cx.string("readView"), JsFunction::new(&mut cx, read_view)?);
        set!(this, &mut cx, cx.string("readCString"), JsFunction::new(&mut cx, read_cstring)?);
        set!(this, &mut cx, cx.string("readString"), JsFunction::new(&mut cx, read_string)?);
        set!(this, &mut cx, cx.string("writeU8"), JsFunction::new(&mut cx, write_u8)?);
        set!(this, &mut cx, cx.string("writeI8"), JsFunction::new(&mut cx, write_i8)?);
        set!(this, &mut cx, cx.string("writeU16"), JsFunction::new(&mut cx, write_u16)?);
        set!(this, &mut cx, cx.string("writeI16"), JsFunction::new(&mut cx, write_i16)?);
        set!(this, &mut cx, cx.string("writeU32"), JsFunction::new(&mut cx, write_u32)?);
        set!(this, &mut cx, cx.string("writeI32"), JsFunction::new(&mut cx, write_i32)?);
        set!(this, &mut cx, cx.string("writeCString"), JsFunction::new(&mut cx, write_cstring)?);
        set!(this, &mut cx, cx.string("writeString"), JsFunction::new(&mut cx, write_string)?);
        set!(this, &mut cx, cx.string("free"), JsFunction::new(&mut cx, free)?);

        Ok(this)
    }
}

fn get_head(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    Ok(cx.number(view.head as f64))
}

fn get_offset(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    Ok(cx.number(view.offset as f64))
}

fn read_u8(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let offset = cx.argument_opt(0);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += 1;
        output
    } as *mut u8;
    Ok(cx.number(unsafe { *offset }))
}

fn read_i8(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let offset = cx.argument_opt(0);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += 1;
        output
    } as *mut i8;
    Ok(cx.number(unsafe { *offset }))
}

fn read_u16(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let offset = cx.argument_opt(0);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += 2;
        output
    } as *mut u16;
    Ok(cx.number(unsafe { *offset }))
}

fn read_i16(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let offset = cx.argument_opt(0);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += 2;
        output
    } as *mut i16;
    Ok(cx.number(unsafe { *offset }))
}

fn read_u32(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let offset = cx.argument_opt(0);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += 4;
        output
    } as *mut u32;
    Ok(cx.number(unsafe { *offset }))
}

fn read_i32(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let offset = cx.argument_opt(0);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += 4;
        output
    } as *mut i32;
    Ok(cx.number(unsafe { *offset }))
}

fn read_ptr(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let offset = cx.argument_opt(0);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += std::mem::size_of::<usize>();
        output
    } as *mut usize;
    Ok(cx.number(unsafe { *offset } as f64))
}

fn read_view(mut cx: FunctionContext) -> JsResult<JsObject> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let offset = cx.argument_opt(0);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += std::mem::size_of::<usize>();
        output
    } as *mut usize;
    View::new(cx, unsafe { *offset })
}

fn read_cstring(mut cx: FunctionContext) -> JsResult<JsString> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let is_u16 = cx.argument_opt(0);
    let is_u16 = if let Some(is_u16) = is_u16 {
        let is_u16: Handle<JsBoolean> = is_u16.downcast_or_throw(&mut cx)?;
        is_u16.value(&mut cx)
    } else {
        false
    };
    let offset_arg = cx.argument_opt(1);
    let offset = if let Some(offset) = offset_arg {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut i8;
    let (string, off) = unsafe {
        let cs = if is_u16 { U16CStr::from_ptr_str(offset as *const u16).to_string_lossy() } else { CStr::from_ptr(offset).to_str().unwrap().to_string() };
        let length = cs.bytes().len();
        (cs, length)
    };
    if let None = offset_arg {
        view.offset += off;
    }
    Ok(cx.string(string))
}

fn read_string(mut cx: FunctionContext) -> JsResult<JsString> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let mut view = view.lock().unwrap();
    let is_u16 = cx.argument_opt(1);
    let is_u16 = if let Some(is_u16) = is_u16 {
        let is_u16: Handle<JsBoolean> = is_u16.downcast_or_throw(&mut cx)?;
        is_u16.value(&mut cx)
    } else {
        false
    };
    let length: Handle<JsNumber> = cx.argument(0)?;
    let length = length.value(&mut cx) as usize;
    let offset = cx.argument_opt(2);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        let output = view.head + view.offset;
        view.offset += length;
        output
    } as *mut u8;
    unsafe {
        if is_u16 {
            Ok(cx.string(U16Str::from_ptr(offset as *const u16, length).to_string().unwrap()))
        } else {
            Ok(cx.string(std::str::from_utf8(std::slice::from_raw_parts(offset, length)).unwrap()))
        }
    }
}

fn write_u8(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    let value: Handle<JsNumber> = cx.argument(0)?;
    let offset_arg = cx.argument_opt(1);
    let offset = if let Some(offset) = offset_arg {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut u8;

    unsafe { *offset = value.value(&mut cx) as u8; }

    Ok(cx.undefined())
}

fn write_i8(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    let value: Handle<JsNumber> = cx.argument(0)?;
    let offset_arg = cx.argument_opt(1);
    let offset = if let Some(offset) = offset_arg {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut i8;

    unsafe { *offset = value.value(&mut cx) as i8; }

    Ok(cx.undefined())
}

fn write_u16(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    let value: Handle<JsNumber> = cx.argument(0)?;
    let offset_arg = cx.argument_opt(1);
    let offset = if let Some(offset) = offset_arg {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut u16;

    unsafe { *offset = value.value(&mut cx) as u16; }

    Ok(cx.undefined())
}

fn write_i16(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    let value: Handle<JsNumber> = cx.argument(0)?;
    let offset_arg = cx.argument_opt(1);
    let offset = if let Some(offset) = offset_arg {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut i16;

    unsafe { *offset = value.value(&mut cx) as i16; }

    Ok(cx.undefined())
}

fn write_u32(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    let value: Handle<JsNumber> = cx.argument(0)?;
    let offset_arg = cx.argument_opt(1);
    let offset = if let Some(offset) = offset_arg {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut u32;

    unsafe { *offset = value.value(&mut cx) as u32; }

    Ok(cx.undefined())
}

fn write_i32(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    let value: Handle<JsNumber> = cx.argument(0)?;
    let offset_arg = cx.argument_opt(1);
    let offset = if let Some(offset) = offset_arg {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut i32;

    unsafe { *offset = value.value(&mut cx) as i32; }

    Ok(cx.undefined())
}

fn write_cstring(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    let value: Handle<JsString> = cx.argument(0)?;
    let offset = cx.argument_opt(1);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut u8;

    let mut value = value.value(&mut cx);
    unsafe {
        value.as_mut_ptr().copy_to(offset, value.len() + 1);
        *(offset.add(value.len())) = 0;
    }

    Ok(cx.undefined())
}

fn write_string(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();
    let value: Handle<JsString> = cx.argument(0)?;
    let offset = cx.argument_opt(1);
    let offset = if let Some(offset) = offset {
        let offset: Handle<JsNumber> = offset.downcast_or_throw(&mut cx)?;
        view.head + offset.value(&mut cx) as usize
    } else {
        view.head + view.offset
    } as *mut u8;

    let mut value = value.value(&mut cx);
    unsafe { value.as_mut_ptr().copy_to(offset, value.len()) }

    Ok(cx.undefined())
}

fn allocate(mut cx: FunctionContext) -> JsResult<JsObject> {
    let pointer: Handle<JsNumber> = cx.argument(0)?;
    let pointer = pointer.value(&mut cx);

    View::new(cx, unsafe { crate::bindings::il2cpp_alloc.unwrap()(pointer as usize) } as usize)
}

fn from_pointer(mut cx: FunctionContext) -> JsResult<JsObject> {
    let pointer: Handle<JsNumber> = cx.argument(0)?;
    let pointer = pointer.value(&mut cx);

    View::new(cx, pointer as usize)
}

fn free(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let view: Handle<JsBox<Mutex<View>>> = get!(cx.this(), &mut cx, cx.string("_box")).downcast_or_throw(&mut cx)?;
    let view = view.lock().unwrap();

    unsafe { crate::bindings::il2cpp_free.unwrap()(view.head as usize as *mut c_void) };
    Ok(cx.undefined())
}

pub(crate) fn load_functions<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<()> {
    let global_obj = cx.empty_object();
    set!(global_obj, cx, cx.string("alloc"), JsFunction::new(cx, allocate)?);
    set!(global_obj, cx, cx.string("fromPointer"), JsFunction::new(cx, from_pointer)?);
    set!(cx.global(), cx, cx.string("MemoryView"), global_obj);
    Ok(())
}