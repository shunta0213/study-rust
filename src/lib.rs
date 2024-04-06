use std::rc::Rc;
use std::sync::Mutex;

// prelude means '*' import allowed, it does not pollute namespace
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // if window is None, the game cannot be start so unwrap is enough
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        // get_element_by_id returns Option<Element> and it's ok for JS to use Element as CanvasElement implicitly
        // but Rust, need to declare the Element as a CanvasElement
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    let context = canvas
        .get_context("2d")
        // get_context return Result<Option<Object>>
        // Result<> means a value can be error, Option<> a value can be None
        // so requires unwrap() twice
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

        // draw image
        let image = web_sys::HtmlImageElement::new().unwrap();
        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut x| x.take()) {
                success_tx.send(Ok(()));
            }
        });
        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut x| x.take()) {
                web_sys::console::log_1(&JsValue::from_str(&format!(
                    "Error loading image: {:?}",
                    err
                )));
                error_tx.send(Err(err));
            }
        });

        image.set_src("Idle (1).png");
        image.set_onload(Some((callback.as_ref()).unchecked_ref()));
        image.set_onerror(Some((error_callback.as_ref()).unchecked_ref()));
        // after end of this scope, callback will be dropped without calling forget()
        // intentionally cause memory leak
        callback.forget();

        success_rx.await;
        context.draw_image_with_html_image_element(&image, 100.0, 100.0);
    });

    Ok(())
}
