use crate::collection::ordered::Ordered;
use crate::collection::page::Page;

pub trait OrderedPage: Ordered + Page {}
