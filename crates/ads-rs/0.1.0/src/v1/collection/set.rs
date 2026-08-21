use crate::v1::collection::Map;

pub type Set<T> = dyn Map<T, ()>;
