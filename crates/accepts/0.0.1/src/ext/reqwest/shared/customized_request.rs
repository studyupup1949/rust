/// Wrapper carrying a body and a customization closure.
pub struct CustomizedRequest<RequestBody, F> {
    body: RequestBody,
    customize: F,
}

impl<RequestBody, F> CustomizedRequest<RequestBody, F> {
    /// Creates a new `CustomizedRequest`.
    pub fn new(body: RequestBody, customize: F) -> Self {
        Self { body, customize }
    }

    pub fn into_parts(self) -> (RequestBody, F) {
        (self.body, self.customize)
    }
}
