use activitystreams_vocabulary::{create_object, field_access};

use crate::DiffSide;

create_object! {
    /// A reference to a part of a merge request diff being reviewed.
    ///
    /// # Example (code quote single line)
    ///
    /// ```rust
    /// use activityforge::{CodeQuote, DiffSide, context};
    ///
    /// # fn main() {
    /// let cq_file = "src/main.rs";
    /// let cq_hunk = r#"fn main() {
    ///     println!("hello forgeverse!");
    /// }"#;
    /// let cq_hunk_json = serde_json::to_string(&cq_hunk).unwrap();
    /// let cq_line = 1u64;
    /// let cq_side = DiffSide::Old;
    /// let cq_outdated = false;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "CodeQuote",
    ///   "cqFile": "{cq_file}",
    ///   "cqHunk": {cq_hunk_json},
    ///   "cqLine": {cq_line},
    ///   "cqSide": "{cq_side}",
    ///   "cqOutdated": {cq_outdated}
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let code_quote = CodeQuote::new()
    ///     .with_context_property(context)
    ///     .with_cq_file(cq_file)
    ///     .with_cq_hunk(cq_hunk)
    ///     .with_cq_line(cq_line)
    ///     .with_cq_side(cq_side)
    ///     .with_cq_outdated(cq_outdated);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&code_quote).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<CodeQuote>(json_str.as_str()).unwrap(),
    ///     code_quote
    /// );
    /// # }
    /// ```
    ///
    /// # Example (code quote line range)
    ///
    /// ```rust
    /// use activityforge::{CodeQuote, DiffSide, context};
    ///
    /// # fn main() {
    /// let cq_file = "src/main.rs";
    /// let cq_hunk = r#"fn main() {
    ///     println!("hello forgeverse!");
    /// }"#;
    /// let cq_hunk_json = serde_json::to_string(&cq_hunk).unwrap();
    /// let cq_start = 1u64;
    /// let cq_start_side = DiffSide::New;
    /// let cq_end = 3u64;
    /// let cq_end_side = DiffSide::New;
    /// let cq_outdated = false;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "CodeQuote",
    ///   "cqFile": "{cq_file}",
    ///   "cqHunk": {cq_hunk_json},
    ///   "cqStart": {cq_start},
    ///   "cqStartSide": "{cq_start_side}",
    ///   "cqEnd": {cq_end},
    ///   "cqEndSide": "{cq_end_side}",
    ///   "cqOutdated": {cq_outdated}
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let code_quote = CodeQuote::new()
    ///     .with_context_property(context)
    ///     .with_cq_file(cq_file)
    ///     .with_cq_hunk(cq_hunk)
    ///     .with_cq_start(cq_start)
    ///     .with_cq_start_side(cq_start_side)
    ///     .with_cq_end(cq_end)
    ///     .with_cq_end_side(cq_end_side)
    ///     .with_cq_outdated(cq_outdated);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&code_quote).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<CodeQuote>(json_str.as_str()).unwrap(),
    ///     code_quote
    /// );
    /// # }
    /// ```
    CodeQuote: crate::ObjectType::CodeQuote {
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_hunk: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_line: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_side: Option<DiffSide>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_start: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_start_side: Option<DiffSide>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_end: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_end_side: Option<DiffSide>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cq_outdated: Option<bool>,
    }
}

field_access! {
    CodeQuote {
        /// Path, relative to repo root, of the file whose changes are being quoted.
        cq_file: option_deref { &str, String },
        /// A diff hunk from the merge request’s diff.
        cq_hunk: option_deref { &str, String },
    }
}

field_access! {
    CodeQuote {
        /// The line of code being commented.
        ///
        /// It may refer to the old or new version of the file, depending on `cqSide`.
        cq_line: option { u64 },
        /// To which version of the file `cqLine` refers.
        cq_side: option { DiffSide },
        /// The first line in a range of lines of code being commented.
        ///
        /// It may refer to the old or new version of the file, depending on `cqStartSide`.
        cq_start: option { u64 },
        /// To which version of the file `cqStart` refers.
        cq_start_side: option { DiffSide },
        /// The last line in a range of lines of code being commented.
        ///
        /// It may refer to the old or new version of the file, depending on `cqEndSide`.
        cq_end: option { u64 },
        /// To which version of the file `cqEnd` refers.
        cq_end_side: option { DiffSide },
        /// Whether these line(s) have been changed by a later version of the merge request.
        cq_outdated: option { bool },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn test_code_quote_single() {
        let cq_file = "src/main.rs";
        let cq_hunk = r#"fn main() {
    println!("hello forgeverse!");
}"#;
        let cq_hunk_json = serde_json::to_string(&cq_hunk).unwrap();
        let cq_line = 1u64;
        let cq_side = DiffSide::Old;
        let cq_outdated = false;

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "CodeQuote",
  "cqFile": "{cq_file}",
  "cqHunk": {cq_hunk_json},
  "cqLine": {cq_line},
  "cqSide": "{cq_side}",
  "cqOutdated": {cq_outdated}
}}"#
        );

        let context = context::forgefed_context();

        let code_quote = CodeQuote::new()
            .with_context_property(context)
            .with_cq_file(cq_file)
            .with_cq_hunk(cq_hunk)
            .with_cq_line(cq_line)
            .with_cq_side(cq_side)
            .with_cq_outdated(cq_outdated);

        assert_eq!(serde_json::to_string_pretty(&code_quote).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<CodeQuote>(json_str.as_str()).unwrap(),
            code_quote
        );
    }

    #[test]
    fn test_code_quote_range() {
        let cq_file = "src/main.rs";
        let cq_hunk = r#"fn main() {
    println!("hello forgeverse!");
}"#;
        let cq_hunk_json = serde_json::to_string(&cq_hunk).unwrap();
        let cq_start = 1u64;
        let cq_start_side = DiffSide::New;
        let cq_end = 3u64;
        let cq_end_side = DiffSide::New;
        let cq_outdated = false;

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "CodeQuote",
  "cqFile": "{cq_file}",
  "cqHunk": {cq_hunk_json},
  "cqStart": {cq_start},
  "cqStartSide": "{cq_start_side}",
  "cqEnd": {cq_end},
  "cqEndSide": "{cq_end_side}",
  "cqOutdated": {cq_outdated}
}}"#
        );

        let context = context::forgefed_context();

        let code_quote = CodeQuote::new()
            .with_context_property(context)
            .with_cq_file(cq_file)
            .with_cq_hunk(cq_hunk)
            .with_cq_start(cq_start)
            .with_cq_start_side(cq_start_side)
            .with_cq_end(cq_end)
            .with_cq_end_side(cq_end_side)
            .with_cq_outdated(cq_outdated);

        assert_eq!(serde_json::to_string_pretty(&code_quote).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<CodeQuote>(json_str.as_str()).unwrap(),
            code_quote
        );
    }
}
