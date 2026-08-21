/// This is to minimize the amount of matches made in the code
/// Give it a Result<Whatever_you_want_to_return, Error> and the type of the success and it'll
/// Basically unwrap the result if its there and if it isn't it'll return a handled error inside a TypedHttpResponse.
/// Default status code is InternalServerError, if you want something different pass it as the first argument as a u16.
/// If you want to also return the success result, then pass a valid status code u16 as a second argument
/// Sorry for defining the error status code first, it's to maintain uniform order. 
#[allow(unused_macros)]
#[macro_export]
macro_rules! unwrap_or_return_handled_error {
    ( $e:expr, $type_of_resp:ty ) => {
        match $e {
            Ok(value) => value,
            Err(error) => return actix_web_utils::traits::macro_traits::ReturnableErrorShape::convert_to_returnable::<$type_of_resp>(&error, 500)
        }
    };
    ( $error_status_code:literal, $e:expr, $type_of_resp:ty ) => {
        match $e {
            Ok(value) => value,
            Err(error) => return actix_web_utils::traits::macro_traits::ReturnableErrorShape::convert_to_returnable::<$type_of_resp>(&error, $error_status_code)
        }
    };
    ( $error_status_code:literal, $success_status_code:literal, $e:expr, $type_of_resp:ty) => {
        match $e {
            Ok(value) => return actix_web_utils::typed_response::TypedHttpResponse::return_standard_response($success_status_code, value),
            Err(error) => return actix_web_utils::traits::macro_traits::ReturnableErrorShape::convert_to_returnable::<$type_of_resp>(&error, $error_status_code)
        }
    }
}

/// Takes whatever error you supply to it and wraps it in a GenericError<E> if err
#[allow(unused_macros)]
#[macro_export]
macro_rules! wrap_generic_error_in_wrapper {
    ( $e:expr ) => {
        match $e {
            Ok(value) => value,
            Err(error) => actix_web_utils::extensions::generic_error::GenericError::wrap(error),
        }
    }
}