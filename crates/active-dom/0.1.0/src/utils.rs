#![allow(dead_code)]

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// Window
/// 
///
pub fn window() -> web_sys::Window {
    web_sys::window().expect("Expecting a window.")
}

/// Document
/// 
/// 
pub fn document() -> web_sys::Document {
    window().document().expect("Expecting a document.")
}

/// Body
/// 
/// 
pub fn body() -> web_sys::HtmlElement {
    document().body().expect("Expecting a body.")
}

/// Create Element
/// 
/// 
pub fn create_element(name: &str) -> web_sys::Element {
    document()
        .create_element(name)
        .expect("Failed to create element.")
}

/// Create Text
/// 
/// 
pub fn create_text(value: &str) -> web_sys::Text {
    document().create_text_node(value)
}

/// Create Comment
/// 
/// 
pub fn create_comment(value: &str) -> web_sys::Comment {
    document().create_comment(value)
}

/// Add Event Listener
/// 
/// 
pub fn add_event_listener<F>(target: &web_sys::Element, name: &str, cb: F)
where
    F: FnMut(web_sys::Event) + 'static,
{
    let cb = Closure::wrap(Box::new(cb) as Box<dyn FnMut(web_sys::Event) + 'static>);

    target
        .add_event_listener_with_callback(name, cb.as_ref().unchecked_ref())
        .expect("Failed to add event listener.");

    cb.forget();
}

/// Append Child
/// 
/// 
pub fn append_child(target: &web_sys::Element, child: &web_sys::Node) {
    target.append_child(child).expect("Failed to append child.");
}

/// Replace Child
/// 
/// 
pub fn replace_child(target: &web_sys::Element, child: &web_sys::Node) {
    target.replace_with_with_node_1(child).expect("Failed to replace child.")
}

/// Set Attribute
pub fn set_attribute(target: &web_sys::Element, key: &str, value: &str) {
    target
        .set_attribute(key, value)
        .expect("Failed to set attribute");
}
