use syn::{Path, Type, TypePath};

use crate::common::{context::CodegenContext, syn::ext::TypePathConstructExt};

use super::{box_path, pin_path};

pub fn pin_box_path(ctx: &CodegenContext, box_t_type: Type) -> Path {
    pin_path(Type::Path(TypePath::from_path(box_path(ctx, box_t_type))))
}
