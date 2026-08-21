use syn::{
    QSelf, Type,
    token::{As, Gt, Lt},
};

pub trait QSelfConstructExt {
    fn from_path(
        lt_token: Lt,
        ty: Box<Type>,
        position: usize,
        as_token: Option<As>,
        gt_token: Gt,
    ) -> QSelf;

    fn from_ty_position_as(ty: Box<Type>, position: usize, as_token: Option<As>) -> QSelf;
}

impl QSelfConstructExt for QSelf {
    fn from_path(
        lt_token: Lt,
        ty: Box<Type>,
        position: usize,
        as_token: Option<As>,
        gt_token: Gt,
    ) -> QSelf {
        QSelf {
            lt_token,
            ty,
            position,
            as_token,
            gt_token,
        }
    }

    fn from_ty_position_as(ty: Box<Type>, position: usize, as_token: Option<As>) -> QSelf {
        Self::from_path(Lt::default(), ty, position, as_token, Gt::default())
    }
}
