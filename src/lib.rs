// prelude means '*' import allowed, it does not pollute namespace
use rand::prelude::*;
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

    fn draw_triangle(
        context: &web_sys::CanvasRenderingContext2d,
        points: [(f64, f64); 3],
        color: (u8, u8, u8),
    ) {
        let [top, left, right] = points;

        let color_str = format!("rgb({},{},{})", color.0, color.1, color.2);
        context.set_fill_style(&wasm_bindgen::JsValue::from_str(&color_str));

        context.move_to(top.0, left.0);
        context.begin_path();
        context.line_to(left.0, left.1);
        context.line_to(right.0, right.1);
        context.line_to(top.0, top.1);
        context.close_path();
        context.stroke();
        context.fill();
    }

    fn mid_point(point_1: (f64, f64), point_2: (f64, f64)) -> (f64, f64) {
        return ((point_1.0 + point_2.0) / 2.0, (point_1.1 + point_2.1) / 2.0);
    }

    fn sierpinski(
        context: &web_sys::CanvasRenderingContext2d,
        points: [(f64, f64); 3],
        color: (u8, u8, u8),
        depth: u8,
    ) {
        draw_triangle(context, points, color);
        let depth = depth - 1;

        if depth > 0 {
            let mut rng = thread_rng();

            let [top, left, right] = points;
            let left_mid = mid_point(top, left);
            let right_mid = mid_point(top, right);
            let bottom_mid = mid_point(left, right);
            let next_color = (
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );

            sierpinski(context, [top, left_mid, right_mid], next_color, depth);
            sierpinski(context, [left_mid, left, bottom_mid], next_color, depth);
            sierpinski(context, [right_mid, bottom_mid, right], next_color, depth);
        }
    }

    // document can be optional in javascript (at least theoretically)
    // so web_sys::window() returns option<Window>
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

    let first_points = [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)];
    sierpinski(&context, first_points, (255, 255, 255), 10);

    Ok(())
}
