use syn::{
    Token,
    parse::{ParseStream, Result},
};

use super::{default_config_parser, spec::LinearAcceptorSpec};

pub fn default_config_list_parser(input: ParseStream) -> Result<Vec<LinearAcceptorSpec>> {
    let mut out = Vec::new();
    while !input.is_empty() {
        out.push(default_config_parser(input)?);

        if input.is_empty() {
            break;
        }

        input.parse::<Token![,]>().ok();
    }
    Ok(out)
}
