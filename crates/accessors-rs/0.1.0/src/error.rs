pub const ACCESSORS_ON_ENUM_ERROR_MESSAGE: &'static str =
    "Accessors derive macro does not support Enum types.";
pub const ACCESSORS_ON_UNION_ERROR_MESSAGE: &'static str =
    "Accessors derive macro does not support Union types.";

pub const ACCESSORS_ATTR_DATA_ONLY_TAKE_ACCESSORS_ATTR: &'static str =
    "AccessorsAttrData from meta need to be a meta list with the ident accessors.";

pub fn invalid_accessors_arg_error_message(arg: &str) -> String {
    format!("Invalid arg {arg:?} as input for accessors. Try get, get_copy, get_ref or set.")
}

pub fn combine_syn_errors<T>(
    iter: impl Iterator<Item = syn::Result<T>>,
) -> syn::Result<Box<impl Iterator<Item = T>>> {
    let (ok_iter, errors): (Vec<_>, Vec<_>) = iter.partition(syn::Result::is_ok);
    // SAFETY: is called only on ok since it's was partitioned with Result::is_ok.
    // Use of Result::unwrap_err_unchecked is necessary since T does not necessarily implement Debug trait.
    let errors = unsafe { errors.into_iter().map(|err| err.unwrap_err_unchecked()) };

    let ok_iter = ok_iter.into_iter().map(syn::Result::unwrap);

    match errors.reduce(|mut error, current| {
        error.combine(current);
        error
    }) {
        Some(err) => Err(err),
        None => Ok(Box::new(ok_iter)),
    }
}
