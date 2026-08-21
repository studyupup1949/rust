use super::function::{Function, FunctionInputArg, FunctionInputs, FunctionOutputArg};
use crate::context::SYSTEM_NS;

fn mk_fn(
    id: &str,
    uri: &str,
    args: Vec<FunctionInputArg>,
    wildcard: bool,
    output_arg: &FunctionOutputArg,
) -> Function {
    Function {
        id: id.to_owned(),
        ns: vec![SYSTEM_NS.to_string()],
        function_uri: uri.to_owned(),
        input_args: FunctionInputs { args, wildcard },
        output_arg: output_arg.clone(),
    }
}

// build all the "standard" functions
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn standard_functions() -> Vec<Function> {
    let mut f = vec![];
    // example of making this function:
    // function stringEqual = "urn:oasis:names:tc:xacml:1.0:function:string-equal" : string string -> boolean

    // common input arguments
    // atomics
    let atomic_string = FunctionInputArg::Atomic("string".to_owned());
    let atomic_boolean = FunctionInputArg::Atomic("boolean".to_owned());
    let atomic_integer = FunctionInputArg::Atomic("integer".to_owned());
    let atomic_double = FunctionInputArg::Atomic("double".to_owned());
    let atomic_date = FunctionInputArg::Atomic("date".to_owned());
    let atomic_time = FunctionInputArg::Atomic("time".to_owned());
    let atomic_date_time = FunctionInputArg::Atomic("dateTime".to_owned());
    let atomic_day_time_duration = FunctionInputArg::Atomic("dayTimeDuration".to_owned());
    let atomic_year_month_duration = FunctionInputArg::Atomic("yearMonthDuration".to_owned());
    let atomic_any_uri = FunctionInputArg::Atomic("anyURI".to_owned());
    let atomic_x500_name = FunctionInputArg::Atomic("x500Name".to_owned());
    let atomic_rfc822_name = FunctionInputArg::Atomic("rfc822Name".to_owned());
    let atomic_hex_binary = FunctionInputArg::Atomic("hexBinary".to_owned());
    let atomic_base64_binary = FunctionInputArg::Atomic("base64Binary".to_owned());
    let atomic_dns_name = FunctionInputArg::Atomic("dnsName".to_owned());
    let atomic_ip_address = FunctionInputArg::Atomic("ipAddress".to_owned());
    let atomic_xpath = FunctionInputArg::Atomic("xpath".to_owned());
    // bags
    let bag_string = FunctionInputArg::AtomicBag("string".to_owned());
    let bag_boolean = FunctionInputArg::AtomicBag("boolean".to_owned());
    let bag_integer = FunctionInputArg::AtomicBag("integer".to_owned());
    let bag_double = FunctionInputArg::AtomicBag("double".to_owned());
    let bag_date = FunctionInputArg::AtomicBag("date".to_owned());
    let bag_time = FunctionInputArg::AtomicBag("time".to_owned());
    let bag_date_time = FunctionInputArg::AtomicBag("dateTime".to_owned());
    let bag_day_time_duration = FunctionInputArg::AtomicBag("dayTimeDuration".to_owned());
    let bag_year_month_duration = FunctionInputArg::AtomicBag("yearMonthDuration".to_owned());
    let bag_any_uri = FunctionInputArg::AtomicBag("anyURI".to_owned());
    let bag_x500_name = FunctionInputArg::AtomicBag("x500Name".to_owned());
    let bag_rfc822_name = FunctionInputArg::AtomicBag("rfc822Name".to_owned());
    let bag_hex_binary = FunctionInputArg::AtomicBag("hexBinary".to_owned());
    let bag_base64_binary = FunctionInputArg::AtomicBag("base64Binary".to_owned());
    let bag_dns_name = FunctionInputArg::AtomicBag("dnsName".to_owned());
    let bag_ip_address = FunctionInputArg::AtomicBag("ipAddress".to_owned());
    // function
    let function = FunctionInputArg::Function;
    // any*
    let any_atomic_or_bag = FunctionInputArg::AnyAtomicOrBag;
    let bag_any_atomic = FunctionInputArg::AnyAtomicBag;

    // common output arguments
    let atomic_string_out = FunctionOutputArg::Atomic("string".to_owned());
    let atomic_boolean_out = FunctionOutputArg::Atomic("boolean".to_owned());
    let atomic_integer_out = FunctionOutputArg::Atomic("integer".to_owned());
    let atomic_double_out = FunctionOutputArg::Atomic("double".to_owned());
    let atomic_date_out = FunctionOutputArg::Atomic("date".to_owned());
    let atomic_time_out = FunctionOutputArg::Atomic("time".to_owned());
    let atomic_date_time_out = FunctionOutputArg::Atomic("dateTime".to_owned());
    let atomic_day_time_duration_out = FunctionOutputArg::Atomic("dayTimeDuration".to_owned());
    let atomic_year_month_duration_out = FunctionOutputArg::Atomic("yearMonthDuration".to_owned());
    let atomic_any_uri_out = FunctionOutputArg::Atomic("anyURI".to_owned());
    let atomic_x500_name_out = FunctionOutputArg::Atomic("x500Name".to_owned());
    let atomic_rfc822_name_out = FunctionOutputArg::Atomic("rfc822Name".to_owned());
    let atomic_hex_binary_out = FunctionOutputArg::Atomic("hexBinary".to_owned());
    let atomic_base64_binary_out = FunctionOutputArg::Atomic("base64Binary".to_owned());
    let atomic_dns_name_out = FunctionOutputArg::Atomic("dnsName".to_owned());
    let atomic_ip_address_out = FunctionOutputArg::Atomic("ipAddress".to_owned());
    // bag outputs
    let bag_string_out = FunctionOutputArg::AtomicBag("string".to_owned());
    let bag_boolean_out = FunctionOutputArg::AtomicBag("boolean".to_owned());
    let bag_integer_out = FunctionOutputArg::AtomicBag("integer".to_owned());
    let bag_double_out = FunctionOutputArg::AtomicBag("double".to_owned());
    let bag_date_out = FunctionOutputArg::AtomicBag("date".to_owned());
    let bag_time_out = FunctionOutputArg::AtomicBag("time".to_owned());
    let bag_date_time_out = FunctionOutputArg::AtomicBag("dateTime".to_owned());
    let bag_day_time_duration_out = FunctionOutputArg::AtomicBag("dayTimeDuration".to_owned());
    let bag_year_month_duration_out = FunctionOutputArg::AtomicBag("yearMonthDuration".to_owned());
    let bag_any_uri_out = FunctionOutputArg::AtomicBag("anyURI".to_owned());
    let bag_x500_name_out = FunctionOutputArg::AtomicBag("x500Name".to_owned());
    let bag_rfc822_name_out = FunctionOutputArg::AtomicBag("rfc822Name".to_owned());
    let bag_hex_binary_out = FunctionOutputArg::AtomicBag("hexBinary".to_owned());
    let bag_base64_binary_out = FunctionOutputArg::AtomicBag("base64Binary".to_owned());
    let bag_dns_name_out = FunctionOutputArg::AtomicBag("dnsName".to_owned());
    let bag_ip_address_out = FunctionOutputArg::AtomicBag("ipAddress".to_owned());
    // anyAtomic outputs
    let bag_any_atomic_out = FunctionOutputArg::AnyAtomicBag;

    // Equality for standard types
    f.push(mk_fn(
        "stringEqual",
        "urn:oasis:names:tc:xacml:1.0:function:string-equal",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "booleanEqual",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-equal",
        vec![atomic_boolean.clone(), atomic_boolean.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerEqual",
        "urn:oasis:names:tc:xacml:1.0:function:integer-equal",
        vec![atomic_integer.clone(), atomic_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleEqual",
        "urn:oasis:names:tc:xacml:1.0:function:double-equal",
        vec![atomic_double.clone(), atomic_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateEqual",
        "urn:oasis:names:tc:xacml:1.0:function:date-equal",
        vec![atomic_date.clone(), atomic_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeEqual",
        "urn:oasis:names:tc:xacml:1.0:function:time-equal",
        vec![atomic_time.clone(), atomic_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeEqual",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-equal",
        vec![atomic_date_time.clone(), atomic_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationEqual",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-equal",
        vec![
            atomic_day_time_duration.clone(),
            atomic_day_time_duration.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationEqual",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-equal",
        vec![
            atomic_year_month_duration.clone(),
            atomic_year_month_duration.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringEqualIgnoreCase",
        "urn:oasis:names:tc:xacml:3.0:function:string-equal-ignore-case",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURIEqual",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-equal",
        vec![atomic_any_uri.clone(), atomic_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "x500NameEqual",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-equal",
        vec![atomic_x500_name.clone(), atomic_x500_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "rfc822NameEqual",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-equal",
        vec![atomic_rfc822_name.clone(), atomic_rfc822_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "hexBinaryEqual",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-equal",
        vec![atomic_hex_binary.clone(), atomic_hex_binary.clone()],
        false,
        &atomic_boolean_out,
    ));

    f.push(mk_fn(
        "base64BinaryEqual",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-equal",
        vec![atomic_base64_binary.clone(), atomic_base64_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerAdd",
        "urn:oasis:names:tc:xacml:1.0:function:integer-add",
        vec![
            atomic_integer.clone(),
            atomic_integer.clone(),
            atomic_integer.clone(),
        ],
        true,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "doubleAdd",
        "urn:oasis:names:tc:xacml:1.0:function:double-add",
        vec![
            atomic_double.clone(),
            atomic_double.clone(),
            atomic_double.clone(),
        ],
        true,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "integerSubtract",
        "urn:oasis:names:tc:xacml:1.0:function:integer-subtract",
        vec![atomic_integer.clone(), atomic_integer.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "doubleSubtract",
        "urn:oasis:names:tc:xacml:1.0:function:double-subtract",
        vec![atomic_double.clone(), atomic_double.clone()],
        false,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "integerMultiply",
        "urn:oasis:names:tc:xacml:1.0:function:integer-multiply",
        vec![
            atomic_integer.clone(),
            atomic_integer.clone(),
            atomic_integer.clone(),
        ],
        true,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "doubleMultiply",
        "urn:oasis:names:tc:xacml:1.0:function:double-multiply",
        vec![
            atomic_double.clone(),
            atomic_double.clone(),
            atomic_double.clone(),
        ],
        true,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "integerDivide",
        "urn:oasis:names:tc:xacml:1.0:function:integer-divide",
        vec![atomic_integer.clone(), atomic_integer.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "doubleDivide",
        "urn:oasis:names:tc:xacml:1.0:function:double-divide",
        vec![atomic_double.clone(), atomic_double.clone()],
        false,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "integerMod",
        "urn:oasis:names:tc:xacml:1.0:function:integer-mod",
        vec![atomic_integer.clone(), atomic_integer.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "integerAbs",
        "urn:oasis:names:tc:xacml:1.0:function:integer-abs",
        vec![atomic_integer.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "doubleAbs",
        "urn:oasis:names:tc:xacml:1.0:function:double-abs",
        vec![atomic_double.clone()],
        false,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "round",
        "urn:oasis:names:tc:xacml:1.0:function:round",
        vec![atomic_double.clone()],
        false,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "floor",
        "urn:oasis:names:tc:xacml:1.0:function:floor",
        vec![atomic_double.clone()],
        false,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "stringNormalizeSpace",
        "urn:oasis:names:tc:xacml:1.0:function:string-normalize-space",
        vec![atomic_string.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "stringNormalizeToLowerCase",
        "urn:oasis:names:tc:xacml:1.0:function:string-normalize-to-lower-case",
        vec![atomic_string.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "doubleToInteger",
        "urn:oasis:names:tc:xacml:1.0:function:double-to-integer",
        vec![atomic_double.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "integerToDouble",
        "urn:oasis:names:tc:xacml:1.0:function:integer-to-double",
        vec![atomic_integer.clone()],
        false,
        &atomic_double_out,
    ));
    // we use the axiomatics convention to disambiguate "or" from the
    // keyword in targets.  And yes, this can take zero args.
    f.push(mk_fn(
        "orFunction",
        "urn:oasis:names:tc:xacml:1.0:function:or",
        vec![atomic_boolean.clone()],
        true,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "andFunction",
        "urn:oasis:names:tc:xacml:1.0:function:and",
        vec![atomic_boolean.clone()],
        true,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "nOf",
        "urn:oasis:names:tc:xacml:1.0:function:n-of",
        vec![atomic_integer.clone(), atomic_boolean.clone()],
        true,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "not",
        "urn:oasis:names:tc:xacml:1.0:function:not",
        vec![atomic_boolean.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerGreaterThan",
        "urn:oasis:names:tc:xacml:1.0:function:integer-greater-than",
        vec![atomic_integer.clone(), atomic_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerGreaterThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:integer-greater-than-or-equal",
        vec![atomic_integer.clone(), atomic_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerLessThan",
        "urn:oasis:names:tc:xacml:1.0:function:integer-less-than",
        vec![atomic_integer.clone(), atomic_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerLessThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:integer-less-than-or-equal",
        vec![atomic_integer.clone(), atomic_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleGreaterThan",
        "urn:oasis:names:tc:xacml:1.0:function:double-greater-than",
        vec![atomic_double.clone(), atomic_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleGreaterThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:double-greater-than-or-equal",
        vec![atomic_double.clone(), atomic_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleLessThan",
        "urn:oasis:names:tc:xacml:1.0:function:double-less-than",
        vec![atomic_double.clone(), atomic_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleLessThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:double-less-than-or-equal",
        vec![atomic_double.clone(), atomic_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeAddDayTimeDuration",
        "urn:oasis:names:tc:xacml:3.0:function:dateTime-add-dayTimeDuration",
        vec![atomic_date_time.clone(), atomic_day_time_duration.clone()],
        false,
        &atomic_date_time_out,
    ));
    f.push(mk_fn(
        "dateTimeAddYearMonthDuration",
        "urn:oasis:names:tc:xacml:3.0:function:dateTime-add-yearMonthDuration",
        vec![atomic_date_time.clone(), atomic_year_month_duration.clone()],
        false,
        &atomic_date_time_out,
    ));
    f.push(mk_fn(
        "dateTimeSubtractDayTimeDuration",
        "urn:oasis:names:tc:xacml:3.0:function:dateTime-subtract-dayTimeDuration",
        vec![atomic_date_time.clone(), atomic_day_time_duration.clone()],
        false,
        &atomic_date_time_out,
    ));
    f.push(mk_fn(
        "dateTimeSubtractYearMonthDuration",
        "urn:oasis:names:tc:xacml:3.0:function:dateTime-subtract-yearMonthDuration",
        vec![atomic_date_time.clone(), atomic_year_month_duration.clone()],
        false,
        &atomic_date_time_out,
    ));
    f.push(mk_fn(
        "dateAddYearMonthDuration",
        "urn:oasis:names:tc:xacml:3.0:function:date-add-yearMonthDuration",
        vec![atomic_date_time.clone(), atomic_day_time_duration.clone()],
        false,
        &atomic_date_out,
    ));
    f.push(mk_fn(
        "dateSubtractYearMonthDuration",
        "urn:oasis:names:tc:xacml:3.0:function:date-subtract-yearMonthDuration",
        vec![atomic_date.clone(), atomic_year_month_duration.clone()],
        false,
        &atomic_date_out,
    ));
    f.push(mk_fn(
        "stringGreaterThan",
        "urn:oasis:names:tc:xacml:1.0:function:string-greater-than",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringGreaterThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:string-greater-than-or-equal",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringLessThan",
        "urn:oasis:names:tc:xacml:1.0:function:string-less-than",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringLessThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:string-less-than-or-equal",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeGreaterThan",
        "urn:oasis:names:tc:xacml:1.0:function:time-greater-than",
        vec![atomic_time.clone(), atomic_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeGreaterThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:time-greater-than-or-equal",
        vec![atomic_time.clone(), atomic_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeLessThan",
        "urn:oasis:names:tc:xacml:1.0:function:time-less-than",
        vec![atomic_time.clone(), atomic_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeLessThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:time-less-than-or-equal",
        vec![atomic_time.clone(), atomic_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeInRange",
        "urn:oasis:names:tc:xacml:2.0:function:time-in-range",
        vec![atomic_time.clone(), atomic_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeGreaterThan",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-greater-than",
        vec![atomic_date_time.clone(), atomic_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeGreaterThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-greater-than-or-equal",
        vec![atomic_date_time.clone(), atomic_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeLessThan",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-less-than",
        vec![atomic_date_time.clone(), atomic_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeLessThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-less-than-or-equal",
        vec![atomic_date_time.clone(), atomic_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateGreaterThan",
        "urn:oasis:names:tc:xacml:1.0:function:date-greater-than",
        vec![atomic_date.clone(), atomic_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateGreaterThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:date-greater-than-or-equal",
        vec![atomic_date.clone(), atomic_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateLessThan",
        "urn:oasis:names:tc:xacml:1.0:function:date-less-than",
        vec![atomic_date.clone(), atomic_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateLessThanOrEqual",
        "urn:oasis:names:tc:xacml:1.0:function:date-less-than-or-equal",
        vec![atomic_date.clone(), atomic_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:string-one-and-only",
        vec![bag_string.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "stringBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:string-bag-size",
        vec![bag_string.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "stringIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:string-is-in",
        vec![atomic_string.clone(), bag_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringBag",
        "urn:oasis:names:tc:xacml:1.0:function:string-bag",
        vec![atomic_string.clone()],
        true,
        &bag_string_out,
    ));
    f.push(mk_fn(
        "booleanOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-one-and-only",
        vec![bag_boolean.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "booleanBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-bag-size",
        vec![bag_boolean.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "booleanIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-is-in",
        vec![atomic_boolean.clone(), bag_boolean.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "booleanBag",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-bag",
        vec![atomic_boolean.clone()],
        true,
        &bag_boolean_out,
    ));
    f.push(mk_fn(
        "integerOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:integer-one-and-only",
        vec![bag_integer.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "integerBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:integer-bag-size",
        vec![bag_integer.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "integerIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:integer-is-in",
        vec![atomic_integer.clone(), bag_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerBag",
        "urn:oasis:names:tc:xacml:1.0:function:integer-bag",
        vec![atomic_integer.clone()],
        true,
        &bag_integer_out,
    ));
    f.push(mk_fn(
        "doubleOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:double-one-and-only",
        vec![bag_double.clone()],
        false,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "doubleBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:double-bag-size",
        vec![bag_double.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "doubleIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:double-is-in",
        vec![atomic_double.clone(), bag_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleBag",
        "urn:oasis:names:tc:xacml:1.0:function:double-bag",
        vec![atomic_double.clone()],
        true,
        &bag_double_out,
    ));
    f.push(mk_fn(
        "timeOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:time-one-and-only",
        vec![bag_time.clone()],
        false,
        &atomic_time_out,
    ));
    f.push(mk_fn(
        "timeBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:time-bag-size",
        vec![bag_time.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "timeIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:time-is-in",
        vec![atomic_time.clone(), bag_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeBag",
        "urn:oasis:names:tc:xacml:1.0:function:time-bag",
        vec![atomic_time.clone()],
        true,
        &bag_time_out,
    ));
    f.push(mk_fn(
        "dateOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:date-one-and-only",
        vec![bag_date.clone()],
        false,
        &atomic_date_out,
    ));
    f.push(mk_fn(
        "dateBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:date-bag-size",
        vec![bag_date.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "dateIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:date-is-in",
        vec![atomic_date.clone(), bag_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateBag",
        "urn:oasis:names:tc:xacml:1.0:function:date-bag",
        vec![atomic_date.clone()],
        true,
        &bag_date_out,
    ));
    f.push(mk_fn(
        "dateTimeOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-one-and-only",
        vec![bag_date_time.clone()],
        false,
        &atomic_date_time_out,
    ));
    f.push(mk_fn(
        "dateTimeBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-bag-size",
        vec![bag_date_time.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "dateTimeIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-is-in",
        vec![atomic_date_time.clone(), bag_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeBag",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-bag",
        vec![atomic_date_time.clone()],
        true,
        &bag_date_time_out,
    ));
    f.push(mk_fn(
        "anyURIOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-one-and-only",
        vec![bag_any_uri.clone()],
        false,
        &atomic_any_uri_out,
    ));
    f.push(mk_fn(
        "anyURIBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-bag-size",
        vec![bag_any_uri.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "anyURIIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-is-in",
        vec![atomic_any_uri.clone(), bag_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURIBag",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-bag",
        vec![atomic_any_uri.clone()],
        true,
        &bag_any_uri_out,
    ));
    f.push(mk_fn(
        "hexBinaryOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-one-and-only",
        vec![bag_hex_binary.clone()],
        false,
        &atomic_hex_binary_out,
    ));
    f.push(mk_fn(
        "hexBinaryBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-bag-size",
        vec![bag_hex_binary.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "hexBinaryIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-is-in",
        vec![atomic_hex_binary.clone(), bag_hex_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "hexBinaryBag",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-bag",
        vec![atomic_hex_binary.clone()],
        true,
        &bag_hex_binary_out,
    ));
    f.push(mk_fn(
        "base64BinaryOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-one-and-only",
        vec![bag_base64_binary.clone()],
        false,
        &atomic_base64_binary_out,
    ));
    f.push(mk_fn(
        "base64BinaryBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-bag-size",
        vec![bag_base64_binary.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "base64BinaryIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-is-in",
        vec![atomic_base64_binary.clone(), bag_base64_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "base64BinaryBag",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-bag",
        vec![atomic_base64_binary.clone()],
        true,
        &bag_base64_binary_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationOneAndOnly",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-one-and-only",
        vec![bag_day_time_duration.clone()],
        false,
        &atomic_day_time_duration_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationBagSize",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-bag-size",
        vec![bag_day_time_duration.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationIsIn",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-is-in",
        vec![
            atomic_day_time_duration.clone(),
            bag_day_time_duration.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationBag",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-bag",
        vec![atomic_day_time_duration.clone()],
        true,
        &bag_day_time_duration_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationOneAndOnly",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-one-and-only",
        vec![bag_year_month_duration.clone()],
        false,
        &atomic_year_month_duration_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationBagSize",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-bag-size",
        vec![bag_year_month_duration.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationIsIn",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-is-in",
        vec![
            atomic_year_month_duration.clone(),
            bag_year_month_duration.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationBag",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-bag",
        vec![atomic_year_month_duration.clone()],
        true,
        &bag_year_month_duration_out,
    ));
    f.push(mk_fn(
        "x500NameOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-one-and-only",
        vec![bag_x500_name.clone()],
        false,
        &atomic_x500_name_out,
    ));
    f.push(mk_fn(
        "x500NameBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-bag-size",
        vec![bag_x500_name.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "x500NameIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-is-in",
        vec![atomic_x500_name.clone(), bag_x500_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "x500NameBag",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-bag",
        vec![atomic_x500_name.clone()],
        true,
        &bag_x500_name_out,
    ));
    f.push(mk_fn(
        "rfc822NameOneAndOnly",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-one-and-only",
        vec![bag_rfc822_name.clone()],
        false,
        &atomic_rfc822_name_out,
    ));
    f.push(mk_fn(
        "rfc822NameBagSize",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-bag-size",
        vec![bag_rfc822_name.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "rfc822NameIsIn",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-is-in",
        vec![atomic_rfc822_name.clone(), bag_rfc822_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "rfc822NameBag",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-bag",
        vec![atomic_rfc822_name.clone()],
        true,
        &bag_rfc822_name_out,
    ));
    f.push(mk_fn(
        "ipAddressOneAndOnly",
        "urn:oasis:names:tc:xacml:2.0:function:ipAddress-one-and-only",
        vec![bag_ip_address.clone()],
        false,
        &atomic_ip_address_out,
    ));
    f.push(mk_fn(
        "ipAddressBagSize",
        "urn:oasis:names:tc:xacml:2.0:function:ipAddress-bag-size",
        vec![bag_ip_address.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "ipAddressBag",
        "urn:oasis:names:tc:xacml:2.0:function:ipAddress-bag",
        vec![atomic_ip_address.clone()],
        true,
        &bag_ip_address_out,
    ));
    f.push(mk_fn(
        "dnsNameOneAndOnly",
        "urn:oasis:names:tc:xacml:2.0:function:dnsName-one-and-only",
        vec![bag_dns_name.clone()],
        false,
        &atomic_dns_name_out,
    ));
    f.push(mk_fn(
        "dnsNameBagSize",
        "urn:oasis:names:tc:xacml:2.0:function:dnsName-bag-size",
        vec![bag_dns_name.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "dnsNameBag",
        "urn:oasis:names:tc:xacml:2.0:function:dnsName-bag",
        vec![atomic_dns_name.clone()],
        true,
        &bag_dns_name_out,
    ));
    f.push(mk_fn(
        "stringConcatenate",
        "urn:oasis:names:tc:xacml:2.0:function:string-concatenate",
        vec![atomic_string.clone()],
        true,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "booleanFromString",
        "urn:oasis:names:tc:xacml:3.0:function:boolean-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringFromBoolean",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-boolean",
        vec![atomic_boolean.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "integerFromString",
        "urn:oasis:names:tc:xacml:3.0:function:integer-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "stringFromInteger",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-integer",
        vec![atomic_integer.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "doubleFromString",
        "urn:oasis:names:tc:xacml:3.0:function:double-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_double_out,
    ));
    f.push(mk_fn(
        "stringFromDouble",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-double",
        vec![atomic_double.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "timeFromString",
        "urn:oasis:names:tc:xacml:3.0:function:time-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_time_out,
    ));
    f.push(mk_fn(
        "stringFromTime",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-time",
        vec![atomic_time.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "dateFromString",
        "urn:oasis:names:tc:xacml:3.0:function:date-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_date_out,
    ));
    f.push(mk_fn(
        "stringFromDate",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-date",
        vec![atomic_date.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "dateTimeFromString",
        "urn:oasis:names:tc:xacml:3.0:function:dateTime-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_date_time_out,
    ));
    f.push(mk_fn(
        "stringFromDateTime",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-dateTime",
        vec![atomic_date_time.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "anyURIFromString",
        "urn:oasis:names:tc:xacml:3.0:function:anyURI-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_any_uri_out,
    ));
    f.push(mk_fn(
        "stringFromAnyURI",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-anyURI",
        vec![atomic_any_uri.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationFromString",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_day_time_duration_out,
    ));
    f.push(mk_fn(
        "stringFromDayTimeDuration",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-dayTimeDuration",
        vec![atomic_day_time_duration.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationFromString",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_year_month_duration_out,
    ));
    f.push(mk_fn(
        "stringFromYearMonthDuration",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-yearMonthDuration",
        vec![atomic_year_month_duration.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "x500NameFromString",
        "urn:oasis:names:tc:xacml:3.0:function:x500Name-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_x500_name_out,
    ));
    f.push(mk_fn(
        "stringFromX500Name",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-x500Name",
        vec![atomic_x500_name.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "rfc822NameFromString",
        "urn:oasis:names:tc:xacml:3.0:function:rfc822Name-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_rfc822_name_out,
    ));
    f.push(mk_fn(
        "stringFromRfc822Name",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-rfc822Name",
        vec![atomic_rfc822_name.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "ipAddressFromString",
        "urn:oasis:names:tc:xacml:3.0:function:ipAddress-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_ip_address_out,
    ));
    f.push(mk_fn(
        "stringFromIpAddress",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-ipAddress",
        vec![atomic_ip_address.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "dnsNameFromString",
        "urn:oasis:names:tc:xacml:3.0:function:dnsName-from-string",
        vec![atomic_string.clone()],
        false,
        &atomic_dns_name_out,
    ));
    f.push(mk_fn(
        "stringFromDnsName",
        "urn:oasis:names:tc:xacml:3.0:function:string-from-dnsName",
        vec![atomic_dns_name.clone()],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "stringStartsWith",
        "urn:oasis:names:tc:xacml:3.0:function:string-starts-with",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURIStartsWith",
        "urn:oasis:names:tc:xacml:3.0:function:anyURI-starts-with",
        vec![atomic_string.clone(), atomic_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringEndsWith",
        "urn:oasis:names:tc:xacml:3.0:function:string-ends-with",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURIEndsWith",
        "urn:oasis:names:tc:xacml:3.0:function:anyURI-ends-with",
        vec![atomic_string.clone(), atomic_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringContains",
        "urn:oasis:names:tc:xacml:3.0:function:string-contains",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURIContains",
        "urn:oasis:names:tc:xacml:3.0:function:anyURI-contains",
        vec![atomic_string.clone(), atomic_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringSubstring",
        "urn:oasis:names:tc:xacml:3.0:function:string-substring",
        vec![
            atomic_string.clone(),
            atomic_integer.clone(),
            atomic_integer.clone(),
        ],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "anyURISubstring",
        "urn:oasis:names:tc:xacml:3.0:function:anyURI-substring",
        vec![
            atomic_any_uri.clone(),
            atomic_integer.clone(),
            atomic_integer.clone(),
        ],
        false,
        &atomic_string_out,
    ));
    f.push(mk_fn(
        "anyOf",
        "urn:oasis:names:tc:xacml:3.0:function:any-of",
        vec![
            function.clone(),
            any_atomic_or_bag.clone(),
            any_atomic_or_bag.clone(),
        ],
        true,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "allOf",
        "urn:oasis:names:tc:xacml:3.0:function:all-of",
        vec![
            function.clone(),
            any_atomic_or_bag.clone(),
            any_atomic_or_bag.clone(),
        ],
        true,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyOfAny",
        "urn:oasis:names:tc:xacml:3.0:function:any-of-any",
        vec![
            function.clone(),
            any_atomic_or_bag.clone(),
            any_atomic_or_bag.clone(),
        ],
        true,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "allOfAny",
        "urn:oasis:names:tc:xacml:1.0:function:all-of-any",
        vec![
            function.clone(),
            bag_any_atomic.clone(),
            bag_any_atomic.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyOfAll",
        "urn:oasis:names:tc:xacml:1.0:function:any-of-all",
        vec![
            function.clone(),
            bag_any_atomic.clone(),
            bag_any_atomic.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "allOfAll",
        "urn:oasis:names:tc:xacml:1.0:function:all-of-all",
        vec![
            function.clone(),
            bag_any_atomic.clone(),
            bag_any_atomic.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "map",
        "urn:oasis:names:tc:xacml:3.0:function:map",
        vec![
            function.clone(),
            any_atomic_or_bag.clone(),
            any_atomic_or_bag.clone(),
        ],
        true,
        &bag_any_atomic_out,
    ));
    f.push(mk_fn(
        "x500NameMatch",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-match",
        vec![atomic_x500_name.clone(), atomic_x500_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "rfc822NameMatch",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-match",
        vec![atomic_string.clone(), atomic_rfc822_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringRegexpMatch",
        "urn:oasis:names:tc:xacml:1.0:function:string-regexp-match",
        vec![atomic_string.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURIRegexpMatch",
        "urn:oasis:names:tc:xacml:2.0:function:anyURI-regexp-match",
        vec![atomic_string.clone(), atomic_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "ipAddressRegexpMatch",
        "urn:oasis:names:tc:xacml:2.0:function:ipAddress-regexp-match",
        vec![atomic_string.clone(), atomic_ip_address.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dnsNameRegexpMatch",
        "urn:oasis:names:tc:xacml:2.0:function:dnsName-regexp-match",
        vec![atomic_string.clone(), atomic_dns_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "rfc822NameRegexpMatch",
        "urn:oasis:names:tc:xacml:2.0:function:rfc822Name-regexp-match",
        vec![atomic_string.clone(), atomic_rfc822_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "x500NameRegexpMatch",
        "urn:oasis:names:tc:xacml:2.0:function:x500Name-regexp-match",
        vec![atomic_string.clone(), atomic_x500_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "xpathNodeCount",
        "urn:oasis:names:tc:xacml:3.0:function:xpath-node-count",
        vec![atomic_xpath.clone()],
        false,
        &atomic_integer_out,
    ));
    f.push(mk_fn(
        "xpathNodeEqual",
        "urn:oasis:names:tc:xacml:3.0:function:xpath-node-equal",
        vec![atomic_xpath.clone(), atomic_xpath.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "xpathNodeMatch",
        "urn:oasis:names:tc:xacml:3.0:function:xpath-node-match",
        vec![atomic_xpath.clone(), atomic_xpath.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:string-intersection",
        vec![bag_string.clone(), bag_string.clone()],
        false,
        &bag_string_out,
    ));

    f.push(mk_fn(
        "stringAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:string-at-least-one-member-of",
        vec![bag_string.clone(), bag_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringUnion",
        "urn:oasis:names:tc:xacml:1.0:function:string-union",
        vec![bag_string.clone(), bag_string.clone(), bag_string.clone()],
        true,
        &bag_string_out,
    ));
    f.push(mk_fn(
        "stringSubset",
        "urn:oasis:names:tc:xacml:1.0:function:string-subset",
        vec![bag_string.clone(), bag_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "stringSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:string-set-equals",
        vec![bag_string.clone(), bag_string.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "booleanIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-intersection",
        vec![bag_boolean.clone(), bag_boolean.clone()],
        false,
        &bag_boolean_out,
    ));
    f.push(mk_fn(
        "booleanAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-at-least-one-member-of",
        vec![bag_boolean.clone(), bag_boolean.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "booleanUnion",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-union",
        vec![
            bag_boolean.clone(),
            bag_boolean.clone(),
            bag_boolean.clone(),
        ],
        true,
        &bag_boolean_out,
    ));
    f.push(mk_fn(
        "booleanSubset",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-subset",
        vec![bag_boolean.clone(), bag_boolean.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "booleanSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:boolean-set-equals",
        vec![bag_boolean.clone(), bag_boolean.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:integer-intersection",
        vec![bag_integer.clone(), bag_integer.clone()],
        false,
        &bag_integer_out,
    ));
    f.push(mk_fn(
        "integerAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:integer-at-least-one-member-of",
        vec![bag_integer.clone(), bag_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerUnion",
        "urn:oasis:names:tc:xacml:1.0:function:integer-union",
        vec![
            bag_integer.clone(),
            bag_integer.clone(),
            bag_integer.clone(),
        ],
        true,
        &bag_integer_out,
    ));
    f.push(mk_fn(
        "integerSubset",
        "urn:oasis:names:tc:xacml:1.0:function:integer-subset",
        vec![bag_integer.clone(), bag_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "integerSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:integer-set-equals",
        vec![bag_integer.clone(), bag_integer.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:double-intersection",
        vec![bag_double.clone(), bag_double.clone()],
        false,
        &bag_double_out,
    ));
    f.push(mk_fn(
        "doubleAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:double-at-least-one-member-of",
        vec![bag_double.clone(), bag_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleUnion",
        "urn:oasis:names:tc:xacml:1.0:function:double-union",
        vec![bag_double.clone(), bag_double.clone(), bag_double.clone()],
        true,
        &bag_double_out,
    ));
    f.push(mk_fn(
        "doubleSubset",
        "urn:oasis:names:tc:xacml:1.0:function:double-subset",
        vec![bag_double.clone(), bag_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "doubleSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:double-set-equals",
        vec![bag_double.clone(), bag_double.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:time-intersection",
        vec![bag_time.clone(), bag_time.clone()],
        false,
        &bag_time_out,
    ));
    f.push(mk_fn(
        "timeAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:time-at-least-one-member-of",
        vec![bag_time.clone(), bag_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeUnion",
        "urn:oasis:names:tc:xacml:1.0:function:time-union",
        vec![bag_time.clone(), bag_time.clone(), bag_time.clone()],
        true,
        &bag_time_out,
    ));
    f.push(mk_fn(
        "timeSubset",
        "urn:oasis:names:tc:xacml:1.0:function:time-subset",
        vec![bag_time.clone(), bag_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "timeSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:time-set-equals",
        vec![bag_time.clone(), bag_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:date-intersection",
        vec![bag_date.clone(), bag_date.clone()],
        false,
        &bag_date_out,
    ));
    f.push(mk_fn(
        "dateAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:date-at-least-one-member-of",
        vec![bag_date.clone(), bag_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateUnion",
        "urn:oasis:names:tc:xacml:1.0:function:date-union",
        vec![bag_date.clone(), bag_date.clone(), bag_date.clone()],
        true,
        &bag_date_out,
    ));
    f.push(mk_fn(
        "dateSubset",
        "urn:oasis:names:tc:xacml:1.0:function:date-subset",
        vec![bag_date.clone(), bag_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:date-set-equals",
        vec![bag_date.clone(), bag_date.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-intersection",
        vec![bag_date_time.clone(), bag_date_time.clone()],
        false,
        &bag_date_time_out,
    ));
    f.push(mk_fn(
        "dateTimeAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-at-least-one-member-of",
        vec![bag_date_time.clone(), bag_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeUnion",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-union",
        vec![
            bag_date_time.clone(),
            bag_date_time.clone(),
            bag_date_time.clone(),
        ],
        true,
        &bag_date_time_out,
    ));
    f.push(mk_fn(
        "dateTimeSubset",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-subset",
        vec![bag_date_time.clone(), bag_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dateTimeSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:dateTime-set-equals",
        vec![bag_date_time.clone(), bag_date_time.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURIIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-intersection",
        vec![bag_any_uri.clone(), bag_any_uri.clone()],
        false,
        &bag_any_uri_out,
    ));
    f.push(mk_fn(
        "anyURIAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-at-least-one-member-of",
        vec![bag_any_uri.clone(), bag_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURIUnion",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-union",
        vec![
            bag_any_uri.clone(),
            bag_any_uri.clone(),
            bag_any_uri.clone(),
        ],
        true,
        &bag_any_uri_out,
    ));
    f.push(mk_fn(
        "anyURISubset",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-subset",
        vec![bag_any_uri.clone(), bag_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "anyURISetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:anyURI-set-equals",
        vec![bag_any_uri.clone(), bag_any_uri.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "hexBinaryIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-intersection",
        vec![bag_hex_binary.clone(), bag_hex_binary.clone()],
        false,
        &bag_hex_binary_out,
    ));
    f.push(mk_fn(
        "hexBinaryAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-at-least-one-member-of",
        vec![bag_hex_binary.clone(), bag_hex_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "hexBinaryUnion",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-union",
        vec![
            bag_hex_binary.clone(),
            bag_hex_binary.clone(),
            bag_hex_binary.clone(),
        ],
        true,
        &bag_hex_binary_out,
    ));
    f.push(mk_fn(
        "hexBinarySubset",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-subset",
        vec![bag_hex_binary.clone(), bag_hex_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "hexBinarySetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:hexBinary-set-equals",
        vec![bag_hex_binary.clone(), bag_hex_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "base64BinaryIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-intersection",
        vec![bag_base64_binary.clone(), bag_base64_binary.clone()],
        false,
        &bag_base64_binary_out,
    ));
    f.push(mk_fn(
        "base64BinaryAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-at-least-one-member-of",
        vec![bag_base64_binary.clone(), bag_base64_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "base64BinaryUnion",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-union",
        vec![
            bag_base64_binary.clone(),
            bag_base64_binary.clone(),
            bag_base64_binary.clone(),
        ],
        true,
        &bag_base64_binary_out,
    ));
    f.push(mk_fn(
        "base64BinarySubset",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-subset",
        vec![bag_base64_binary.clone(), bag_base64_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "base64BinarySetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:base64Binary-set-equals",
        vec![bag_base64_binary.clone(), bag_base64_binary.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationIntersection",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-intersection",
        vec![bag_day_time_duration.clone(), bag_day_time_duration.clone()],
        false,
        &bag_day_time_duration_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-at-least-one-member-of",
        vec![bag_day_time_duration.clone(), bag_day_time_duration.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationUnion",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-union",
        vec![
            bag_day_time_duration.clone(),
            bag_day_time_duration.clone(),
            bag_day_time_duration.clone(),
        ],
        true,
        &bag_day_time_duration_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationSubset",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-subset",
        vec![bag_day_time_duration.clone(), bag_day_time_duration.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "dayTimeDurationSetEquals",
        "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-set-equals",
        vec![bag_day_time_duration.clone(), bag_day_time_duration.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationIntersection",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-intersection",
        vec![
            bag_year_month_duration.clone(),
            bag_year_month_duration.clone(),
        ],
        false,
        &bag_year_month_duration_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-at-least-one-member-of",
        vec![
            bag_year_month_duration.clone(),
            bag_year_month_duration.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationUnion",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-union",
        vec![
            bag_year_month_duration.clone(),
            bag_year_month_duration.clone(),
            bag_year_month_duration.clone(),
        ],
        true,
        &bag_year_month_duration_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationSubset",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-subset",
        vec![
            bag_year_month_duration.clone(),
            bag_year_month_duration.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "yearMonthDurationSetEquals",
        "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-set-equals",
        vec![
            bag_year_month_duration.clone(),
            bag_year_month_duration.clone(),
        ],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "x500NameIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-intersection",
        vec![bag_x500_name.clone(), bag_x500_name.clone()],
        false,
        &bag_x500_name_out,
    ));
    f.push(mk_fn(
        "x500NameAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-at-least-one-member-of",
        vec![bag_x500_name.clone(), bag_x500_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "x500NameUnion",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-union",
        vec![
            bag_x500_name.clone(),
            bag_x500_name.clone(),
            bag_x500_name.clone(),
        ],
        true,
        &bag_x500_name_out,
    ));
    f.push(mk_fn(
        "x500NameSubset",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-subset",
        vec![bag_x500_name.clone(), bag_x500_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "x500NameSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:x500Name-set-equals",
        vec![bag_x500_name.clone(), bag_x500_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "rfc822NameIntersection",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-intersection",
        vec![bag_rfc822_name.clone(), bag_rfc822_name.clone()],
        false,
        &bag_rfc822_name_out,
    ));
    f.push(mk_fn(
        "rfc822NameAtLeastOneMemberOf",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-at-least-one-member-of",
        vec![bag_rfc822_name.clone(), bag_rfc822_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "rfc822NameUnion",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-union",
        vec![
            bag_rfc822_name.clone(),
            bag_rfc822_name.clone(),
            bag_rfc822_name.clone(),
        ],
        true,
        &bag_rfc822_name_out,
    ));
    f.push(mk_fn(
        "rfc822NameSubset",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-subset",
        vec![bag_rfc822_name.clone(), bag_rfc822_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "rfc822NameSetEquals",
        "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-set-equals",
        vec![bag_rfc822_name.clone(), bag_rfc822_name.clone()],
        false,
        &atomic_boolean_out,
    ));
    f.push(mk_fn(
        "accessPermitted",
        "urn:oasis:names:tc:xacml:3.0:function:access-permitted",
        vec![atomic_any_uri.clone(), atomic_string.clone()],
        false,
        &atomic_boolean_out,
    ));

    f
}
