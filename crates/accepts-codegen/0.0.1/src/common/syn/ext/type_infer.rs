use syn::{TypeInfer, token::Underscore};

pub trait TypeInferConstructExt {
    fn new_default() -> TypeInfer;
}

impl TypeInferConstructExt for TypeInfer {
    fn new_default() -> TypeInfer {
        TypeInfer {
            underscore_token: Underscore::default(),
        }
    }
}
