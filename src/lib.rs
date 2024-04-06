use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::convert::IntoWasmAbi;
// prelude means '*' import allowed, it does not pollute namespace
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
        let window = web_sys::window().unwrap();
        // cast Javascript Promise to Rust Future
        let resp_value = wasm_bindgen_futures::JsFuture::from(
            // calls window.fetch(json_path)
            window.fetch_with_str(json_path),
        )
        // await the Future (Promise)
        // '?' operator is used to propagate error
        // when error occurs, it returns Err(err) and the function returns Err(err)
        // otherwise, it returns Ok(value) and the value is assigned to resp_value
        .await?;
        // cast JsValue to Response
        // note '?' operator is used
        let resp: web_sys::Response = resp_value.dyn_into()?;

        wasm_bindgen_futures::JsFuture::from(resp.json()?).await
    }

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
        let json = fetch_json("rhb.json").await.unwrap();
        let sheet: Sheet = serde_wasm_bindgen::from_value(json).expect("failed to parse JSON");

        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);

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

        image.set_src("rhb.png");
        image.set_onload(Some((callback.as_ref()).unchecked_ref()));
        image.set_onerror(Some((error_callback.as_ref()).unchecked_ref()));
        // after end of this scope, callback will be dropped without calling forget()
        // intentionally cause memory leak
        callback.forget();
        success_rx.await;

        let mut frame = -1;
        let interval_callback = Closure::wrap(Box::new(move || {
            frame = (frame + 1) % 8;
            let frame_name = format!("Run ({}).png", frame + 1);
            context.clear_rect(0.0, 0.0, 600.0, 600.0);
            let sprite = sheet.frames.get(&frame_name).expect("failed to get sprite");
            context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                sprite.frame.x.into(),
                sprite.frame.y.into(),
                sprite.frame.w.into(),
                sprite.frame.h.into(),
                300.0,
                300.0,
                sprite.frame.w.into(),
                sprite.frame.h.into(),
            );
        }) as Box<dyn FnMut()>);
        window.set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            50,
        );
        interval_callback.forget();
    });

    Ok(())
}
