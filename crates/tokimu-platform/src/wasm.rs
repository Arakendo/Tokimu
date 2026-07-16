use crate::{PlatformInputEvent, PlatformResult, WindowConfig};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use tokimu_input::{KeyCode, MouseButton};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

#[cfg(target_arch = "wasm32")]
use web_sys::{window, Document, HtmlCanvasElement, KeyboardEvent, MouseEvent, Window};

#[cfg(target_arch = "wasm32")]
fn boxed_error(message: impl Into<String>) -> Box<dyn std::error::Error> {
    std::io::Error::other(message.into()).into()
}

pub fn backend_name() -> &'static str {
    "wasm"
}

#[cfg(target_arch = "wasm32")]
pub fn install_browser_input_bridge<F>(config: WindowConfig, event_handler: F) -> PlatformResult<()>
where
    F: FnMut(PlatformInputEvent) + 'static,
{
    let window = window().ok_or_else(|| boxed_error("browser window is not available"))?;
    let document = window
        .document()
        .ok_or_else(|| boxed_error("browser document is not available"))?;
    let canvas = ensure_canvas(&document, &config)?;

    document.set_title(&config.title);
    canvas.set_width(config.width.max(1));
    canvas.set_height(config.height.max(1));

    let event_handler = Rc::new(RefCell::new(event_handler));
    emit_event(
        &event_handler,
        PlatformInputEvent::Resized {
            width: canvas.width(),
            height: canvas.height(),
        },
    );

    attach_keyboard_listeners(&window, event_handler.clone())?;
    attach_mouse_listeners(&document, &canvas, event_handler.clone())?;
    attach_resize_listener(&window, &canvas, event_handler.clone())?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn ensure_canvas(document: &Document, config: &WindowConfig) -> PlatformResult<HtmlCanvasElement> {
    if let Some(element) = document.get_element_by_id("tokimu-canvas") {
        return element
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| boxed_error("tokimu-canvas exists but is not a canvas element"));
    }

    let canvas = document
        .create_element("canvas")
        .map_err(|error| boxed_error(format!("{:?}", error)))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| boxed_error("failed to create a canvas element"))?;
    canvas.set_id("tokimu-canvas");
    canvas.set_width(config.width.max(1));
    canvas.set_height(config.height.max(1));

    let body = document
        .body()
        .ok_or_else(|| boxed_error("browser document has no body"))?;
    body.append_child(&canvas)
        .map_err(|error| boxed_error(format!("{:?}", error)))?;

    Ok(canvas)
}

#[cfg(target_arch = "wasm32")]
fn attach_keyboard_listeners(
    window: &Window,
    event_handler: Rc<RefCell<impl FnMut(PlatformInputEvent) + 'static>>,
) -> PlatformResult<()> {
    let document_for_escape = window
        .document()
        .ok_or_else(|| boxed_error("browser document is not available"))?;
    let keydown_handler = event_handler.clone();
    let keydown = Closure::wrap(Box::new(move |event: web_sys::Event| {
        if let Ok(key_event) = event.dyn_into::<KeyboardEvent>() {
            if key_event.code() == "Escape" && document_for_escape.pointer_lock_element().is_some()
            {
                document_for_escape.exit_pointer_lock();
            }
            if let Some(key) = map_key_code(&key_event.code()) {
                emit_event(
                    &keydown_handler,
                    PlatformInputEvent::KeyboardInput { key, pressed: true },
                );
            }
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    let keyup_handler = event_handler;
    let keyup = Closure::wrap(Box::new(move |event: web_sys::Event| {
        if let Ok(key_event) = event.dyn_into::<KeyboardEvent>() {
            if let Some(key) = map_key_code(&key_event.code()) {
                emit_event(
                    &keyup_handler,
                    PlatformInputEvent::KeyboardInput {
                        key,
                        pressed: false,
                    },
                );
            }
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    window
        .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
        .map_err(|error| boxed_error(format!("{:?}", error)))?;
    window
        .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())
        .map_err(|error| boxed_error(format!("{:?}", error)))?;
    keydown.forget();
    keyup.forget();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn attach_mouse_listeners(
    document: &Document,
    canvas: &HtmlCanvasElement,
    event_handler: Rc<RefCell<impl FnMut(PlatformInputEvent) + 'static>>,
) -> PlatformResult<()> {
    let move_handler = event_handler.clone();
    let document_for_move = document.clone();
    let cursor_moved = Closure::wrap(Box::new(move |event: web_sys::Event| {
        if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
            if document_for_move.pointer_lock_element().is_some() {
                emit_event(
                    &move_handler,
                    PlatformInputEvent::MouseMotion {
                        delta_x: mouse_event.movement_x() as f32,
                        delta_y: mouse_event.movement_y() as f32,
                    },
                );
            } else {
                emit_event(
                    &move_handler,
                    PlatformInputEvent::CursorMoved {
                        x: mouse_event.offset_x() as f32,
                        y: mouse_event.offset_y() as f32,
                    },
                );
            }
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    let button_handler = event_handler;
    let canvas_for_button = canvas.clone();
    let mouse_button = Closure::wrap(Box::new(move |event: web_sys::Event| {
        if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
            if let Some(button) = map_mouse_button(mouse_event.button()) {
                if mouse_event.type_() == "mousedown" && mouse_event.button() == 0 {
                    let _ = canvas_for_button.request_pointer_lock();
                }
                emit_event(
                    &button_handler,
                    PlatformInputEvent::MouseInput {
                        button,
                        pressed: event_type_indicates_press(mouse_event.type_().as_str()),
                    },
                );
            }
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    canvas
        .add_event_listener_with_callback("mousemove", cursor_moved.as_ref().unchecked_ref())
        .map_err(|error| boxed_error(format!("{:?}", error)))?;
    canvas
        .add_event_listener_with_callback("mousedown", mouse_button.as_ref().unchecked_ref())
        .map_err(|error| boxed_error(format!("{:?}", error)))?;
    canvas
        .add_event_listener_with_callback("mouseup", mouse_button.as_ref().unchecked_ref())
        .map_err(|error| boxed_error(format!("{:?}", error)))?;

    let document_for_lock_state = document.clone();
    let canvas_for_lock_state = canvas.clone();
    let pointer_lock_change = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        let locked = document_for_lock_state.pointer_lock_element().is_some();
        let _ = canvas_for_lock_state
            .style()
            .set_property("cursor", if locked { "none" } else { "default" });
    }) as Box<dyn FnMut(web_sys::Event)>);

    document
        .add_event_listener_with_callback(
            "pointerlockchange",
            pointer_lock_change.as_ref().unchecked_ref(),
        )
        .map_err(|error| boxed_error(format!("{:?}", error)))?;

    let _ = canvas.style().set_property("cursor", "default");

    cursor_moved.forget();
    mouse_button.forget();
    pointer_lock_change.forget();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn attach_resize_listener(
    window: &Window,
    canvas: &HtmlCanvasElement,
    event_handler: Rc<RefCell<impl FnMut(PlatformInputEvent) + 'static>>,
) -> PlatformResult<()> {
    let window_for_resize = window.clone();
    let resize_handler = event_handler;
    let canvas = canvas.clone();
    let resize = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        let width = window_for_resize
            .inner_width()
            .ok()
            .and_then(|value| value.as_f64())
            .map(|value| value.max(1.0) as u32)
            .unwrap_or_else(|| canvas.width().max(1));
        let height = window_for_resize
            .inner_height()
            .ok()
            .and_then(|value| value.as_f64())
            .map(|value| value.max(1.0) as u32)
            .unwrap_or_else(|| canvas.height().max(1));
        canvas.set_width(width);
        canvas.set_height(height);
        emit_event(
            &resize_handler,
            PlatformInputEvent::Resized { width, height },
        );
    }) as Box<dyn FnMut(web_sys::Event)>);

    window
        .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())
        .map_err(|error| boxed_error(format!("{:?}", error)))?;
    resize.forget();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn emit_event<F>(handler: &Rc<RefCell<F>>, event: PlatformInputEvent)
where
    F: FnMut(PlatformInputEvent) + 'static,
{
    (handler.borrow_mut())(event);
}

#[cfg(target_arch = "wasm32")]
fn map_key_code(code: &str) -> Option<KeyCode> {
    match code {
        "Escape" => Some(KeyCode::Escape),
        "Space" => Some(KeyCode::Space),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyA" => Some(KeyCode::KeyA),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyZ" => Some(KeyCode::KeyZ),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
fn map_mouse_button(button: i16) -> Option<MouseButton> {
    match button {
        0 => Some(MouseButton::Left),
        1 => Some(MouseButton::Middle),
        2 => Some(MouseButton::Right),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
fn event_type_indicates_press(event_type: &str) -> bool {
    event_type == "mousedown"
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn install_browser_input_bridge<F>(
    _config: WindowConfig,
    _event_handler: F,
) -> PlatformResult<()>
where
    F: FnMut(PlatformInputEvent) + 'static,
{
    Err("browser input bridge is only available on wasm32 targets".into())
}
