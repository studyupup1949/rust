//! # Ad Hoc Result
//! 
//! A library providing an extension to Rust's standard `Result` type that
//! allows for "ad hoc" values to be provided alongside errors.
//! 
//! This is useful in scenarios where a computation may fail but can still
//! recommend a reasonable value to use despite the failure.  
//! For example, this is needed when you can solve a linear system, but the accuracy is poor due to large condition numbers: you may want to return the computed solution as a recommendation, even though the operation is technically a failure.
//! This is an interface to recommend a value anyway when computation fails.

/// An enum representing either success (`Ok`), failure with a recommended value (`AdHoc`),
/// or complete failure (`Err`).
/// 
/// # Type Parameters
/// 
/// * `T` - The type of the value in the success case or recommended value
/// * `E` - The type of the error value
/// 
/// # Examples
/// 
/// ```
/// use ad_hoc_result::AdHocResult;
/// 
/// // Simulate a function that returns AdHocResult
/// fn divide(a: f64, b: f64) -> AdHocResult<f64, String> {
///     if b == 0.0 {
///         AdHocResult::AdHoc(f64::INFINITY, "Division by zero".to_string())
///     } else {
///         AdHocResult::Ok(a / b)
///     }
/// }
/// 
/// let result = divide(10.0, 2.0);
/// assert_eq!(result.unwrap(), 5.0);
/// 
/// let ad_hoc_result = divide(10.0, 0.0);
/// assert_eq!(ad_hoc_result.unwrap_adhoc(), f64::INFINITY);
/// ```
pub enum AdHocResult<T, E> {
    /// Contains the success value
    Ok(T),
    /// Contains both a recommended value and an error explaining why the computation failed
    AdHoc(T,E),
    /// Contains only the error value
    Err(E)
}

impl<T, E> AdHocResult<T, E> {
    /// Unwraps a result, yielding the content of an `Ok`.
    ///
    /// # Panics
    ///
    /// Panics with the provided message if the value is an `AdHoc` or `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let x: AdHocResult<u32, &str> = AdHocResult::Ok(2);
    /// assert_eq!(x.expect("Testing expect"), 2);
    /// ```
    pub fn expect(self, message: &str) -> T {
        match self {
            AdHocResult::Ok(x) => x,
            _ => panic!("{}", message)
        }
    }

    /// Unwraps a result, yielding the content of an `Ok`.
    ///
    /// # Panics
    ///
    /// Panics with a generic "Unwrap fails" message if the value is an `AdHoc` or `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let x: AdHocResult<u32, &str> = AdHocResult::Ok(2);
    /// assert_eq!(x.unwrap(), 2);
    /// ```
    pub fn unwrap(self) -> T {
        self.expect("Unwrap fails")
    }

    /// Unwraps a result, yielding the content of an `Ok` or `AdHoc`.
    ///
    /// # Panics
    ///
    /// Panics with the provided message if the value is an `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let x: AdHocResult<u32, &str> = AdHocResult::AdHoc(2, "Not ideal");
    /// assert_eq!(x.expect_adhoc("Testing expect_adhoc"), 2);
    /// ```
    pub fn expect_adhoc(self, message: &str) -> T {
        match self {
            AdHocResult::Ok(x) => x,
            AdHocResult::AdHoc(x, _) => x,
            _ => panic!("{}", message)
        }
    }

    /// Unwraps a result, yielding the content of an `Ok` or `AdHoc`.
    ///
    /// # Panics
    ///
    /// Panics with a generic "Unwrap fails" message if the value is an `Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let x: AdHocResult<u32, &str> = AdHocResult::AdHoc(2, "Not ideal");
    /// assert_eq!(x.unwrap_adhoc(), 2);
    /// ```
    pub fn unwrap_adhoc(self) -> T {
        self.expect_adhoc("Unwrap fails")
    }

    /// Converts the `AdHocResult<T, E>` into a `Result<T, E>`.
    ///
    /// This conversion treats both `Err` and `AdHoc` variants as errors,
    /// but for `AdHoc` it will discard the recommended value and only keep the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let ok: AdHocResult<i32, &str> = AdHocResult::Ok(42);
    /// assert_eq!(ok.to_result(), Ok(42));
    ///
    /// let adhoc: AdHocResult<i32, &str> = AdHocResult::AdHoc(42, "Not perfect");
    /// assert_eq!(adhoc.to_result(), Err("Not perfect"));
    ///
    /// let err: AdHocResult<i32, &str> = AdHocResult::Err("Error");
    /// assert_eq!(err.to_result(), Err("Error"));
    /// ```
    pub fn to_result(self) -> Result<T, E> {
        match self {
            AdHocResult::Ok(v) => Ok(v),
            AdHocResult::AdHoc(_, e) => Err(e),
            AdHocResult::Err(e) => Err(e),
        }
    }

    /// Converts the `AdHocResult<T, E>` into a `Result<T, E>` treating `AdHoc` as success.
    ///
    /// This conversion treats only `Err` variant as an error, while both `Ok` and `AdHoc`
    /// variants are treated as success, prioritizing the value but discarding any error
    /// information from `AdHoc`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let ok: AdHocResult<i32, &str> = AdHocResult::Ok(42);
    /// assert_eq!(ok.to_result_with_adhoc(), Ok(42));
    ///
    /// let adhoc: AdHocResult<i32, &str> = AdHocResult::AdHoc(42, "Not perfect");
    /// assert_eq!(adhoc.to_result_with_adhoc(), Ok(42));
    ///
    /// let err: AdHocResult<i32, &str> = AdHocResult::Err("Error");
    /// assert_eq!(err.to_result_with_adhoc(), Err("Error"));
    /// ```
    pub fn to_result_with_adhoc(self) -> Result<T, E> {
        match self {
            AdHocResult::Ok(v) => Ok(v),
            AdHocResult::AdHoc(v, _) => Ok(v),
            AdHocResult::Err(e) => Err(e),
        }
    }
}

impl<T, E> From<Result<T, E>> for AdHocResult<T, E> {
    /// Converts a `Result<T, E>` into an `AdHocResult<T, E>`.
    ///
    /// This conversion maps `Ok` variants to `AdHocResult::Ok` and
    /// `Err` variants to `AdHocResult::Err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let ok_result: Result<i32, &str> = Ok(42);
    /// let adhoc_ok: AdHocResult<i32, &str> = ok_result.into();
    /// 
    /// match adhoc_ok {
    ///     AdHocResult::Ok(v) => assert_eq!(v, 42),
    ///     _ => panic!("Expected AdHocResult::Ok"),
    /// }
    ///
    /// let err_result: Result<i32, &str> = Err("Error");
    /// let adhoc_err: AdHocResult<i32, &str> = err_result.into();
    ///
    /// match adhoc_err {
    ///     AdHocResult::Err(e) => assert_eq!(e, "Error"),
    ///     _ => panic!("Expected AdHocResult::Err"),
    /// }
    /// ```
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(v) => AdHocResult::Ok(v),
            Err(e) => AdHocResult::Err(e),
        }
    }
}

impl<T, E> From<AdHocResult<T, E>> for Result<T, E> {
    /// Converts an `AdHocResult<T, E>` into a `Result<T, E>`.
    ///
    /// This conversion maps `AdHocResult::Ok` to `Ok` and both
    /// `AdHocResult::AdHoc` and `AdHocResult::Err` to `Err`.
    /// For `AdHoc`, the recommended value is discarded.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let adhoc_ok: AdHocResult<i32, &str> = AdHocResult::Ok(42);
    /// let ok_result: Result<i32, &str> = adhoc_ok.into();
    /// assert_eq!(ok_result, Ok(42));
    ///
    /// let adhoc: AdHocResult<i32, &str> = AdHocResult::AdHoc(42, "Not perfect");
    /// let adhoc_result: Result<i32, &str> = adhoc.into();
    /// assert_eq!(adhoc_result, Err("Not perfect"));
    ///
    /// let adhoc_err: AdHocResult<i32, &str> = AdHocResult::Err("Error");
    /// let err_result: Result<i32, &str> = adhoc_err.into();
    /// assert_eq!(err_result, Err("Error"));
    /// ```
    fn from(adhoc: AdHocResult<T, E>) -> Self {
        adhoc.to_result()
    }
}

// Additional helper trait implementation
impl<T, E> AdHocResult<T, E> {
    /// Creates a new `AdHocResult` in the `Ok` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let x: AdHocResult<u32, &str> = AdHocResult::new_ok(42);
    /// assert_eq!(x.unwrap(), 42);
    /// ```
    pub fn new_ok(value: T) -> Self {
        AdHocResult::Ok(value)
    }

    /// Creates a new `AdHocResult` in the `AdHoc` variant with a recommended value and an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let x: AdHocResult<u32, &str> = AdHocResult::new_adhoc(42, "Not ideal");
    /// assert_eq!(x.unwrap_adhoc(), 42);
    /// ```
    pub fn new_adhoc(value: T, error: E) -> Self {
        AdHocResult::AdHoc(value, error)
    }

    /// Creates a new `AdHocResult` in the `Err` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use ad_hoc_result::AdHocResult;
    ///
    /// let x: AdHocResult<u32, &str> = AdHocResult::new_err("Error occurred");
    /// assert!(matches!(x, AdHocResult::Err(_)));
    /// ```
    pub fn new_err(error: E) -> Self {
        AdHocResult::Err(error)
    }
}
