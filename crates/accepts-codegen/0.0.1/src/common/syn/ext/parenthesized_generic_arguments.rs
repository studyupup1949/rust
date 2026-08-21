use syn::{
    ParenthesizedGenericArguments, ReturnType, Type,
    punctuated::Punctuated,
    token::{Comma, Paren},
};

pub trait ParenthesizedGenericArgumentsConstructExt {
    fn from_parts(
        paren_token: Paren,
        inputs: Punctuated<Type, Comma>,
        output: ReturnType,
    ) -> ParenthesizedGenericArguments;

    fn from_inputs_output(
        inputs: Punctuated<Type, Comma>,
        output: ReturnType,
    ) -> ParenthesizedGenericArguments;
}

impl ParenthesizedGenericArgumentsConstructExt for ParenthesizedGenericArguments {
    fn from_parts(
        paren_token: Paren,
        inputs: Punctuated<Type, Comma>,
        output: ReturnType,
    ) -> ParenthesizedGenericArguments {
        Self {
            paren_token,
            inputs,
            output,
        }
    }

    fn from_inputs_output(
        inputs: Punctuated<Type, Comma>,
        output: ReturnType,
    ) -> ParenthesizedGenericArguments {
        <Self as ParenthesizedGenericArgumentsConstructExt>::from_parts(
            Paren::default(),
            inputs,
            output,
        )
    }
}
