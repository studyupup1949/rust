use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserInfo {
    pub user_agent: String,
    pub accept_header: String,
    pub language: String,
    pub color_depth: u32,
    pub screen_height: u32,
    pub screen_width: u32,
    pub time_zone_offset: i32,
    pub java_enabled: bool,
}
