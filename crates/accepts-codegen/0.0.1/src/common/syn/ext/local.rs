use syn::{
    Attribute, Local, LocalInit, Pat,
    token::{Let, Semi},
};

pub trait LocalConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        let_token: Let,
        pat: Pat,
        init: Option<LocalInit>,
        semi_token: Semi,
    ) -> Local;

    fn from_pat_init(pat: Pat, init: Option<LocalInit>) -> Local;

    fn from_pat(pat: Pat) -> Local;
}

impl LocalConstructExt for Local {
    fn from_parts(
        attrs: Vec<Attribute>,
        let_token: Let,
        pat: Pat,
        init: Option<LocalInit>,
        semi_token: Semi,
    ) -> Local {
        Local {
            attrs,
            let_token,
            pat,
            init,
            semi_token,
        }
    }

    fn from_pat_init(pat: Pat, init: Option<LocalInit>) -> Local {
        Self::from_parts(Vec::new(), Let::default(), pat, init, Semi::default())
    }

    fn from_pat(pat: Pat) -> Local {
        Self::from_pat_init(pat, None)
    }
}
