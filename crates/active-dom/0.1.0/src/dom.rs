use super::{
    add_event_listener, append_child, body, create_effect, create_element, create_text,
    replace_child, set_attribute, Runtime,
};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

/// DOM
///
/// DOm Element builder.
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct DOM(web_sys::Element);

impl DOM {
    /// New
    /// 
    /// New instance.
    pub fn new(name: &str) -> Self {
        let ele = create_element(name);
        Self(ele)
    }

    /// Attr
    /// 
    /// Set attribute.
    pub fn attr(self, key: &str, value: &str) -> Self {
        set_attribute(&self, key, value);
        self
    }

    /// On
    /// 
    /// Add event listener.
    pub fn on<F>(self, name: &str, cb: F) -> Self
    where
        F: FnMut(web_sys::Event) + 'static,
    {
        add_event_listener(&self, name, cb);
        self
    }

    /// Child
    /// 
    /// Add children.
    pub fn child(self, child: &DOM) -> Self {
        append_child(&self, child);
        self
    }

    /// Text
    /// 
    /// Add text.
    pub fn text(self, value: &str) -> Self {
        let text = create_text(value);
        append_child(&self, &text);

        self
    }

    /// Dyn Text
    /// 
    /// Add reactive text.
    pub fn dyn_text<F>(self, ctx: &'static Runtime, cb: F) -> Self
    where
        F: 'static + Fn() -> String,
    {
        let text = create_text("not dyn.");
        append_child(&self, &text);

        create_effect(ctx, move || {
            let output = cb();
            text.set_data(&output);
        });

        self
    }

    /// Dyn Child
    /// 
    /// Add reactive children.
    pub fn dyn_child<F>(self, ctx: &'static Runtime, func: F) -> Self
    where
        F: 'static + Fn() -> DOM,
    {
        let child = DOM::new("span");
        append_child(&self, &child);

        let wrapper = Rc::new(RefCell::new(child));

        create_effect(ctx, move || {
            let output = func();

            let mut child = wrapper.borrow_mut();
            replace_child(&child, &output);
            *child = output;
        });

        self
    }

    /// Dyn Attr
    /// 
    /// Add reactive attribute.
    pub fn dyn_attr<F>(self, ctx: &'static Runtime, key: &str, func: F) -> Self
    where
        F: 'static + Fn() -> String,
    {
        let key = key.to_string();
        let this_dom = self.clone();

        create_effect(ctx, move || {
            set_attribute(&this_dom, &key, &func());
        });

        self
    }
}

impl Deref for DOM {
    type Target = web_sys::Element;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


/// Mount
/// 
/// Mount to html body.
pub fn mount(cb: impl FnOnce(&'static Runtime) -> DOM) {
    let ctx: &'static Runtime = Box::leak(Box::default());

    let body = body();
    let output = cb(ctx);
    append_child(&body, &output);
}
