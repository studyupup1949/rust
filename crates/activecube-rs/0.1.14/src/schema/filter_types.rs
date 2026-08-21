use async_graphql::dynamic::*;

pub fn build_filter_primitives() -> Vec<InputObject> {
    vec![
        InputObject::new("IntFilter")
            .description("Filter conditions for integer fields (values passed as strings)")
            .field(InputValue::new("eq", TypeRef::named(TypeRef::STRING)).description("Equal to"))
            .field(InputValue::new("ne", TypeRef::named(TypeRef::STRING)).description("Not equal to"))
            .field(InputValue::new("gt", TypeRef::named(TypeRef::STRING)).description("Greater than"))
            .field(InputValue::new("ge", TypeRef::named(TypeRef::STRING)).description("Greater than or equal to"))
            .field(InputValue::new("lt", TypeRef::named(TypeRef::STRING)).description("Less than"))
            .field(InputValue::new("le", TypeRef::named(TypeRef::STRING)).description("Less than or equal to"))
            .field(InputValue::new("in", TypeRef::named_nn_list(TypeRef::STRING)).description("Matches any value in the list"))
            .field(InputValue::new("notIn", TypeRef::named_nn_list(TypeRef::STRING)).description("Does not match any value in the list"))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN)).description("True to match NULL, false to match non-NULL")),
        InputObject::new("FloatFilter")
            .description("Filter conditions for floating-point fields (values passed as strings)")
            .field(InputValue::new("eq", TypeRef::named(TypeRef::STRING)).description("Equal to"))
            .field(InputValue::new("ne", TypeRef::named(TypeRef::STRING)).description("Not equal to"))
            .field(InputValue::new("gt", TypeRef::named(TypeRef::STRING)).description("Greater than"))
            .field(InputValue::new("ge", TypeRef::named(TypeRef::STRING)).description("Greater than or equal to"))
            .field(InputValue::new("lt", TypeRef::named(TypeRef::STRING)).description("Less than"))
            .field(InputValue::new("le", TypeRef::named(TypeRef::STRING)).description("Less than or equal to"))
            .field(InputValue::new("in", TypeRef::named_nn_list(TypeRef::STRING)).description("Matches any value in the list"))
            .field(InputValue::new("notIn", TypeRef::named_nn_list(TypeRef::STRING)).description("Does not match any value in the list"))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN)).description("True to match NULL, false to match non-NULL")),
        InputObject::new("StringFilter")
            .description("Filter conditions for string fields")
            .field(InputValue::new("is", TypeRef::named(TypeRef::STRING)).description("Exact match"))
            .field(InputValue::new("not", TypeRef::named(TypeRef::STRING)).description("Not equal to"))
            .field(InputValue::new("in", TypeRef::named_nn_list(TypeRef::STRING)).description("Matches any value in the list"))
            .field(InputValue::new("notIn", TypeRef::named_nn_list(TypeRef::STRING)).description("Does not match any value in the list"))
            .field(InputValue::new("like", TypeRef::named(TypeRef::STRING)).description("SQL LIKE pattern match (use % as wildcard)"))
            .field(InputValue::new("includes", TypeRef::named(TypeRef::STRING)).description("Contains substring (case-sensitive)"))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN)).description("True to match NULL, false to match non-NULL")),
        InputObject::new("RelativeTimeInput")
            .description("Relative time offset from now. Specify one field.")
            .field(InputValue::new("minutes_ago", TypeRef::named(TypeRef::INT)).description("Minutes ago from now"))
            .field(InputValue::new("hours_ago", TypeRef::named(TypeRef::INT)).description("Hours ago from now"))
            .field(InputValue::new("days_ago", TypeRef::named(TypeRef::INT)).description("Days ago from now")),
        InputObject::new("DateTimeFilter")
            .description("Filter conditions for datetime fields. Use format: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SSZ")
            .field(InputValue::new("is", TypeRef::named(TypeRef::STRING)).description("Exact datetime match"))
            .field(InputValue::new("not", TypeRef::named(TypeRef::STRING)).description("Not equal to"))
            .field(InputValue::new("after", TypeRef::named(TypeRef::STRING)).description("Strictly after (>)"))
            .field(InputValue::new("before", TypeRef::named(TypeRef::STRING)).description("Strictly before (<)"))
            .field(InputValue::new("since", TypeRef::named(TypeRef::STRING)).description("On or after (>=)"))
            .field(InputValue::new("till", TypeRef::named(TypeRef::STRING)).description("On or before (<=)"))
            .field(InputValue::new("since_relative", TypeRef::named("RelativeTimeInput")).description("On or after a relative time (e.g. {hours_ago: 24})"))
            .field(InputValue::new("till_relative", TypeRef::named("RelativeTimeInput")).description("On or before a relative time"))
            .field(InputValue::new("after_relative", TypeRef::named("RelativeTimeInput")).description("Strictly after a relative time"))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN)).description("True to match NULL, false to match non-NULL")),
        InputObject::new("BoolFilter")
            .description("Filter conditions for boolean fields")
            .field(InputValue::new("eq", TypeRef::named(TypeRef::BOOLEAN)).description("Equal to"))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN)).description("True to match NULL, false to match non-NULL")),
        InputObject::new("DecimalFilter")
            .description("Filter conditions for high-precision decimal fields (values passed as strings to preserve precision)")
            .field(InputValue::new("eq", TypeRef::named(TypeRef::STRING)).description("Equal to"))
            .field(InputValue::new("ne", TypeRef::named(TypeRef::STRING)).description("Not equal to"))
            .field(InputValue::new("gt", TypeRef::named(TypeRef::STRING)).description("Greater than"))
            .field(InputValue::new("ge", TypeRef::named(TypeRef::STRING)).description("Greater than or equal to"))
            .field(InputValue::new("lt", TypeRef::named(TypeRef::STRING)).description("Less than"))
            .field(InputValue::new("le", TypeRef::named(TypeRef::STRING)).description("Less than or equal to"))
            .field(InputValue::new("in", TypeRef::named_nn_list(TypeRef::STRING)).description("Matches any value in the list"))
            .field(InputValue::new("notIn", TypeRef::named_nn_list(TypeRef::STRING)).description("Does not match any value in the list"))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN)).description("True to match NULL, false to match non-NULL")),
    ]
}

pub fn build_limit_input() -> InputObject {
    InputObject::new("LimitInput")
        .description("Pagination control for query results")
        .field(InputValue::new("count", TypeRef::named(TypeRef::INT)).description("Maximum number of records to return"))
        .field(InputValue::new("offset", TypeRef::named(TypeRef::INT)).description("Number of records to skip"))
}
