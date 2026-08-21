use serde::{Deserialize, Serialize};

use crate::ToHtml;
use super::{
    list::{bullet_list::BulletList, ordered_list::OrderedList},
    code_block::CodeBlock, 
    media::{media_group::MediaGroup, media_single::MediaSingle},
    paragraph::Paragraph, 
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blockquote {
    pub content: Vec<Content>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Content {
    BulletList(BulletList),
    CodeBlock(CodeBlock),
    MediaGroup(MediaGroup),
    MediaSingle(MediaSingle),
    OrderedList(OrderedList),
    Paragraph(Paragraph),
}

impl ToHtml for Blockquote {
    fn to_html(&self) -> String {
        let mut content = String::new();

        for node in &self.content {
            content.push_str(&node.to_html());
        }
        
        format!(r#"<blockquote style = "border-left: 2px solid rgba(9, 30, 66, 0.14); padding-left: 16px; margin-top: 12px;">{content}</blockquote>"#)
    }
}

impl ToHtml for Content {
    fn to_html(&self) -> String {
        match self {
            Content::BulletList(bullet_list) => bullet_list.to_html(),
            Content::CodeBlock(code_block) => code_block.to_html(),
            Content::MediaGroup(media_group) => media_group.to_html(),
            Content::MediaSingle(media_single) => media_single.to_html(),
            Content::OrderedList(ordered_list) => ordered_list.to_html(),
            Content::Paragraph(paragraph) => paragraph.to_html(),
        }
    }
}

impl Blockquote {
    pub(crate) fn replace_media_urls(&mut self, urls: &mut Vec<String>) {
        for content in self.content.iter_mut() {
            content.replace_media_urls(urls);
        }        
    }
}

impl Content {
    pub(crate) fn replace_media_urls(&mut self, urls: &mut Vec<String>) {
        match self {
            Content::BulletList(bullet_list) => bullet_list.replace_media_urls(urls),
            Content::CodeBlock(_code_block) => (),
            Content::MediaGroup(media_group) => media_group.replace_media_urls(urls),
            Content::MediaSingle(media_single) => media_single.replace_media_urls(urls),
            Content::OrderedList(ordered_list) => ordered_list.replace_media_urls(urls),
            Content::Paragraph(_paragraph) => (),
        }
    }
}
