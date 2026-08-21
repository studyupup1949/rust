/// A `Message` to send with a [`Client`][crate::Client].
///
/// To configure a `Message`, use [`Message::builder()`].
#[derive(serde::Serialize)]
pub struct Message {
    da: String,
    ud: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    costcentre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    udh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    srr: Option<String>,
}

impl Message {
    /// Creates a `MessageBuilder` to configure a `Message`.
    pub fn builder() -> MessageBuilder {
        MessageBuilder {
            ..MessageBuilder::default()
        }
    }
}

/// A builder to construct the properties of a [`Message`].
#[derive(Default, Debug)]
pub struct MessageBuilder {
    da: Option<String>,
    ud: Option<String>,
    limit: Option<u8>,
    costcentre: Option<String>,
    private: Option<bool>,
    oa: Option<String>,
    udh: Option<String>,
    srr: Option<String>,
}

impl MessageBuilder {
    /// Returns a `Message` that uses this `MessageBuilder` configuration.
    pub fn build(self) -> Message {
        Message {
            da: self
                .da
                .expect("Cannot build sms without destination, `da`."),
            ud: self.ud.expect("Cannot build sms without message, `ud`."),
            limit: self.limit,
            costcentre: self.costcentre,
            private: self.private,
            oa: self.oa,
            udh: self.udh,
            srr: self.srr,
        }
    }

    /// Sets the `da`, or Destination Address, for the `Message`.
    ///
    /// This is the number to which the message is to be sent and should be a
    /// full international format number (however, national format is also
    /// accepted). This may be a SIP2SIM ICCID to send direct to a SIM.
    ///
    /// This is the same as `MessageBuilder::destination()`
    pub fn da(mut self, da: impl Into<String>) -> Self {
        self.da = Some(da.into());
        self
    }

    /// Sets the `da`, or Destination Address, for the `Message`.
    ///
    /// This is the same as `MessageBuilder::da()`
    pub fn destination(self, destination: impl Into<String>) -> Self {
        self.da(destination)
    }

    /// Sets the `ud`, or User Data ie. the text body, for the `Message`.
    ///
    /// This is the message to send, encoded in UTF-8.
    ///
    /// This is the same as `MessageBuilder::message()`
    pub fn ud(mut self, ud: impl Into<String>) -> Self {
        self.ud = Some(ud.into());
        self
    }

    /// Sets the `ud`, or User Data ie. the text body, for the `Message`.
    ///
    /// This is the same as `MessageBuilder::ud()`
    pub fn message(self, message: impl Into<String>) -> Self {
        self.ud(message)
    }

    /// Sets the `limit` for the `Message`.
    ///
    /// This is the limit on the number of parts that the message may be sent
    /// in.
    pub fn limit(mut self, limit: u8) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the `costcentre` for the `Message`.
    ///
    /// This is the optional, up to 10 characters, code that is included in the
    /// bill XML data.
    pub fn costcentre(mut self, costcentre: impl Into<String>) -> Self {
        self.costcentre = Some(costcentre.into());
        self
    }

    /// Sets the `private` setting for the `Message`.
    ///
    /// This marks the message as private, such that it is not included in the
    /// itemised billing.
    pub fn private(mut self, private: bool) -> Self {
        self.private = Some(private);
        self
    }

    /// Sets the `oa`, or originator, for the `Message`.
    ///
    /// This is used to set where the message is from. Normally this is not
    /// needed as it will default to your `username` phone number.
    pub fn oa(mut self, oa: impl Into<String>) -> Self {
        self.oa = Some(oa.into());
        self
    }

    /// Sets the `udh`, or User Data Header, for the `Message`.
    ///
    /// This is a hex-coded GSM 03.40 UDH.
    pub fn udh(mut self, udh: impl Into<String>) -> Self {
        self.udh = Some(udh.into());
        self
    }

    /// Sets the `srr`, or SMS Status Report, for the `Message`.
    ///
    /// This is the URL or email address to send delivery reports.
    pub fn srr(mut self, srr: impl Into<String>) -> Self {
        self.srr = Some(srr.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_default_all_none() {
        let mb = Message::builder();
        assert_eq!(mb.da, None);
        assert_eq!(mb.ud, None);
        assert_eq!(mb.limit, None);
        assert_eq!(mb.costcentre, None);
        assert_eq!(mb.private, None);
        assert_eq!(mb.oa, None);
        assert_eq!(mb.udh, None);
        assert_eq!(mb.srr, None);
    }

    #[test]
    fn basic_builder_setters_work() {
        let mb = Message::builder().da("123");
        assert_eq!(mb.da, Some(String::from("123")));

        let mb = Message::builder().ud("123");
        assert_eq!(mb.ud, Some(String::from("123")));
        assert_eq!(mb.da, None);

        let mb = Message::builder().limit(1);
        assert_eq!(mb.limit, Some(1));
        assert_eq!(mb.ud, None);

        let mb = Message::builder().costcentre("123");
        assert_eq!(mb.costcentre, Some(String::from("123")));
        assert_eq!(mb.limit, None);

        let mb = Message::builder().private(true);
        assert_eq!(mb.private, Some(true));
        assert_eq!(mb.costcentre, None);

        let mb = Message::builder().oa("123");
        assert_eq!(mb.oa, Some(String::from("123")));
        assert_eq!(mb.private, None);

        let mb = Message::builder().udh("123");
        assert_eq!(mb.udh, Some(String::from("123")));
        assert_eq!(mb.oa, None);

        let mb = Message::builder().srr("123");
        assert_eq!(mb.srr, Some(String::from("123")));
        assert_eq!(mb.udh, None);
    }

    #[test]
    fn basic_builder_works() {
        let message = Message::builder().da("123").ud("456").build();
        assert_eq!(message.da, "123");
        assert_eq!(message.ud, "456");
    }

    #[test]
    fn sugar_builder_works() {
        let message = Message::builder().destination("123").message("456").build();
        assert_eq!(message.da, "123");
        assert_eq!(message.ud, "456");
    }

    #[test]
    fn basic_builder_order_doesnt_matter() {
        let message1 = Message::builder().da("123").ud("456").build();
        let message2 = Message::builder().ud("456").da("123").build();
        assert_eq!(message1.da, message2.da);
        assert_eq!(message1.ud, message2.ud);
    }

    #[test]
    fn sugar_or_basic_builder_doesnt_matter() {
        let message1 = Message::builder().da("123").message("456").build();
        let message2 = Message::builder().destination("123").ud("456").build();
        assert_eq!(message1.da, message2.da);
        assert_eq!(message1.ud, message2.ud);
    }

    #[test]
    #[should_panic]
    fn build_without_da_fails() {
        Message::builder().ud("123").build();
    }

    #[test]
    #[should_panic]
    fn build_without_ud_fails() {
        Message::builder().da("123").build();
    }
}
