use crate::AJMainWindow;
use slint::{ToSharedString, Weak};

pub fn show_alert(
    weak: Weak<AJMainWindow>,
    msg: impl ToSharedString + Send + 'static,
    typ: crate::AJAlertType,
) {
    let msg = msg.to_shared_string();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = weak.upgrade() {
            ui.invoke_show_alert(msg, typ);
        }
    })
    .unwrap();
}
