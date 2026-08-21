use crate::AdicShapeError;
use leptos::prelude::document;


// Modified from https://github.com/thaw-ui/thaw/blob/main/thaw_utils/src/dom/mount_style.rs
// NOTE: This may not work for ssr
pub fn mount_style(id: &str, content: &'static str) -> Result<(), AdicShapeError> {
    let id = format!("adic-id-{id}");

    let head = document().head()
        .ok_or(AdicShapeError::StyleError("head does not exist".to_string()))?;
    let style = head
        .query_selector(&format!("style#{id}"))
        .map_err(|_| AdicShapeError::StyleError("query style element error".to_string()))?;

    if style.is_none() {

        let style = document()
            .create_element("style")
            .map_err(|_| AdicShapeError::StyleError("create style element error".to_string()))?;
        _ = style.set_attribute("id", &id);
        style.set_text_content(Some(content));
        _ = head.prepend_with_node_1(&style);

    }

    Ok(())

}
