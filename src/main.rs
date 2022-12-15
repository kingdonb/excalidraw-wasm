use lib::{
    painter::Painter,
    store::AppState,
    view::toolbar::Toolbar,
    widget::{create_widget, shape::Rect, WidgetKind},
};
use sycamore::prelude::*;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{FontFace, HtmlCanvasElement, KeyboardEvent, MouseEvent};

fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    let _ = FontFace::new_with_str("Virgil".into(), "url(https://uploads.codesandbox.io/uploads/user/ed077012-e728-4a42-8395-cbd299149d62/AflB-FG_Virgil.ttf)")
        .unwrap()
        .load();

    sycamore::render(|ctx| view!(ctx, App()));
}

#[component]
fn App<G: Html>(ctx: BoundedScope) -> View<G> {
    let window = web_sys::window().expect("no global `window` exists");
    let window_width = window.inner_width().unwrap().as_f64().unwrap();
    let window_height = window.inner_height().unwrap().as_f64().unwrap();

    let canvas_ref: &NodeRef<G> = create_node_ref(ctx);
    let painter = Painter::new();

    let drawing_state = create_signal(ctx, (0, 0, 0));

    let app_state = AppState {
        selected_kind: create_rc_signal(WidgetKind::Rectangle),
        elements: create_rc_signal(vec![]),
    };
    let app_state = provide_context(ctx, app_state);
    let is_mounted = create_signal(ctx, false);

    on_mount(ctx, || {
        let window = web_sys::window().expect("should have a window in this context");
        let app_state_cloned = app_state.clone();
        let handler = move |event: KeyboardEvent| {
            if event.key() == "Backspace" {
                app_state_cloned.delete_selected_elements();
            }
        };
        let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);

        window
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
        is_mounted.set(true);
    });

    create_effect(ctx, move || {
        if *is_mounted.get() {
            let elements = app_state.elements.get();
            painter.draw_elements(canvas_ref, elements);
        }
    });

    view! (ctx,
        div {
            Toolbar()
            canvas(
                ref=canvas_ref,
                class="fixed top-10 left-0",
                width=window_width,
                height=window_height,
                id="canvas",
                on:mousedown= move |event|  {
                    let id = app_state.add_element();
                    let mouse_event = event.dyn_into::<MouseEvent>().unwrap();
                    let x = mouse_event.offset_x();
                    let y = mouse_event.offset_y();

                    if *app_state.selected_kind.get() == WidgetKind::Text {
                        let (rect, text) = get_text_info(canvas_ref,x,y);
                        app_state.update_element(id, rect, vec![text]);
                        return;
                    }
                    // tracing::info!("Mouse down at ({}, {})", x, y);
                    drawing_state.set((id, x, y));
                },
                on:mousemove= move |event| {
                    let (id, start_x, start_y) = *drawing_state.get();
                    if id > 0 {
                        let mouse_event = event.dyn_into::<MouseEvent>().unwrap();
                        let x = mouse_event.offset_x();
                        let y = mouse_event.offset_y();
                        let widget = create_widget(*app_state.selected_kind.get(), Rect::new(start_x, start_y, x, y));
                        let config_string = widget.get_config(&painter);
                        app_state.update_element(id, Rect::new(start_x, start_y, x, y), config_string);
                    }
                },
                on:mouseup= move |event| {
                    drawing_state.set((0, 0, 0));
                    let mouse_event = event.dyn_into::<MouseEvent>().unwrap();
                    let x = mouse_event.offset_x();
                    let y = mouse_event.offset_y();
                    tracing::info!("Mouse up at ({}, {})", x, y);
                    app_state.delete_selection_element();
                },
                on:keydown= move |event| {
                    tracing::info!("Key down at ({:?})", event);
                }
            )
        }
    )
}

pub fn get_text_info<G: Html>(canvas_ref: &NodeRef<G>, x: i32, y: i32) -> (Rect, String) {
    let canvas: HtmlCanvasElement = canvas_ref.get::<DomNode>().unchecked_into();
    let window = web_sys::window().expect("should have a window in this context");
    let ctx = canvas
        .get_context("2d")
        .expect("should get context")
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .expect("should cast to context");
    let text = window
        .prompt_with_message("What text do you want?")
        .unwrap();
    let text = text.unwrap();
    let text_measure = ctx.measure_text(&text).unwrap();

    let height = text_measure.font_bounding_box_ascent() + text_measure.font_bounding_box_descent();
    let width = text_measure.width();
    let rect = Rect {
        start_x: x,
        start_y: y,
        end_x: x + width as i32,
        end_y: y + height as i32,
    };
    (rect, text)
}
