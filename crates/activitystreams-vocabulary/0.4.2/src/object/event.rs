use crate::create_object;

create_object! {
    /// Represents any kind of event.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{DateTime, Event, Name};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Going-Away Party for Jim").unwrap();
    /// let start_time = "2014-12-31T23:00:00-08:00";
    /// let end_time = "2015-01-01T06:00:00-08:00";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Event",
    ///   "name": "{name}",
    ///   "startTime": "{start_time}",
    ///   "endTime": "{end_time}"
    /// }}"#);
    ///
    /// let event = Event::new()
    ///     .with_name(name)
    ///     .with_start_time(start_time.parse::<DateTime>().unwrap())
    ///     .with_end_time(end_time.parse::<DateTime>().unwrap());
    ///
    /// assert_eq!(serde_json::to_string_pretty(&event).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Event>(json_str.as_str()).unwrap(),
    ///     event
    /// );
    /// # }
    /// ```
    Event: ObjectType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DateTime, Name};

    #[test]
    fn test_valid() {
        let name = Name::try_from("Going-Away Party for Jim").unwrap();
        let start_time = "2014-12-31T23:00:00-08:00";
        let end_time = "2015-01-01T06:00:00-08:00";

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Event",
  "name": "{name}",
  "startTime": "{start_time}",
  "endTime": "{end_time}"
}}"#
        );

        let event = Event::new()
            .with_name(name)
            .with_start_time(start_time.parse::<DateTime>().unwrap())
            .with_end_time(end_time.parse::<DateTime>().unwrap());

        assert_eq!(serde_json::to_string_pretty(&event).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Event>(json_str.as_str()).unwrap(),
            event
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Event>(json_str).is_err());
    }
}
