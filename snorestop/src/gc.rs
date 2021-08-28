use nodejs::neon::prelude::{NeonResult, Context, FunctionContext, JsResult, JsBox, Finalize, Handle, JsFunction, Object, Channel, Root};
use crate::set;

struct GcHandle {
    callback: Root<JsFunction>,
    channel: Channel
}

impl Finalize for GcHandle {
    fn finalize<'a, C: Context<'a>>(self, _: &mut C) {
        let channel = self.channel.clone();
        println!("finalizing something");
        channel.send(|mut cx| {
            let undef = cx.undefined();
            self.callback.into_inner(&mut cx).call(&mut cx, undef, vec![undef])?;

            Ok(())
        });
    }
}

fn create_gc_handle(mut cx: FunctionContext) -> JsResult<JsBox<GcHandle>> {
    let free: Handle<JsFunction> = cx.argument(0)?;
    let free = free.root(&mut cx);
    let channel = cx.channel();
    Ok(
        JsBox::new(&mut cx, GcHandle {
            callback: free,
            channel
        })
    )
}

pub(crate) fn load_functions<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<()> {
    set!(cx.global(), cx, cx.string("create_js_gc_handle"), JsFunction::new(cx, create_gc_handle)?);
    Ok(())
}