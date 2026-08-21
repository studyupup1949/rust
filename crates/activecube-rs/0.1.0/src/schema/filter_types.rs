use async_graphql::dynamic::*;

pub fn build_filter_primitives() -> Vec<InputObject> {
    vec![
        InputObject::new("IntFilter")
            .field(InputValue::new("eq", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("ne", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("gt", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("ge", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("lt", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("le", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("in", TypeRef::named_nn_list(TypeRef::INT)))
            .field(InputValue::new("notIn", TypeRef::named_nn_list(TypeRef::INT)))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN))),
        InputObject::new("FloatFilter")
            .field(InputValue::new("eq", TypeRef::named(TypeRef::FLOAT)))
            .field(InputValue::new("ne", TypeRef::named(TypeRef::FLOAT)))
            .field(InputValue::new("gt", TypeRef::named(TypeRef::FLOAT)))
            .field(InputValue::new("ge", TypeRef::named(TypeRef::FLOAT)))
            .field(InputValue::new("lt", TypeRef::named(TypeRef::FLOAT)))
            .field(InputValue::new("le", TypeRef::named(TypeRef::FLOAT)))
            .field(InputValue::new("in", TypeRef::named_nn_list(TypeRef::FLOAT)))
            .field(InputValue::new("notIn", TypeRef::named_nn_list(TypeRef::FLOAT)))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN))),
        InputObject::new("StringFilter")
            .field(InputValue::new("is", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("not", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("in", TypeRef::named_nn_list(TypeRef::STRING)))
            .field(InputValue::new("notIn", TypeRef::named_nn_list(TypeRef::STRING)))
            .field(InputValue::new("like", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("includes", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN))),
        InputObject::new("DateTimeFilter")
            .field(InputValue::new("is", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("not", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("after", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("before", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("since", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("till", TypeRef::named(TypeRef::STRING)))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN))),
        InputObject::new("BoolFilter")
            .field(InputValue::new("eq", TypeRef::named(TypeRef::BOOLEAN)))
            .field(InputValue::new("isNull", TypeRef::named(TypeRef::BOOLEAN))),
    ]
}

pub fn build_limit_input() -> InputObject {
    InputObject::new("LimitInput")
        .field(InputValue::new("count", TypeRef::named(TypeRef::INT)))
        .field(InputValue::new("offset", TypeRef::named(TypeRef::INT)))
}
