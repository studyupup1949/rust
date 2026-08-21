use crate::Acs37800ReadError;

pub(crate) fn assert_is_bus_error(err: &Acs37800ReadError) {
    #[cfg(feature = "std")]
    assert!(
        matches!(err, Acs37800ReadError::Io(_)),
        "unexpected error: {err:?}"
    );

    #[cfg(not(feature = "std"))]
    assert!(
        matches!(err, Acs37800ReadError::Io),
        "unexpected error: {err:?}"
    );
}
