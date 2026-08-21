use crate::Domain;

#[test]
fn is_valid_label() {
    let test_cases: &[(&str, bool, bool)] = &[
        ("", false, false),
        ("a", false, true),
        ("azAZ09", true, true),
        ("azAZ09", false, false),
        ("/", true, false),
        (":", true, false),
        ("@", true, false),
        ("[", true, false),
        ("`", true, false),
        ("{", true, false),
        ("-", true, false),
        ("a-", true, false),
        ("-a", true, false),
        ("a--a", true, false),
        ("a-a", true, true),
        ("a-a-a", true, true),
    ];
    for (label, ignore_case, expected) in test_cases {
        let result: bool = Domain::is_valid_label(label.as_bytes(), *ignore_case);
        let result_str: bool = Domain::is_valid_label_str(*label, *ignore_case);
        assert_eq!(result, result_str);
        assert_eq!(result, *expected);
    }
}

#[test]
fn is_valid_label_len() {
    let mut label: String = String::new();
    for _ in 0..63 {
        label.push('a');
    }
    assert_eq!(Domain::is_valid_label(label.as_bytes(), false), true);
    label.push('a');
    assert_eq!(Domain::is_valid_label(label.as_bytes(), false), false);
}

#[test]
fn is_valid_name() {
    let test_cases: &[(&str, bool, bool)] = &[
        ("", false, false),
        ("a", true, true),
        ("A", true, true),
        ("A", false, false),
        ("a.", true, false),
        (".a", true, false),
        ("a..a", true, false),
        ("a.a.a", false, true),
    ];
    for (name, ignore_case, expected) in test_cases {
        let result: bool = Domain::is_valid_name(name.as_bytes(), *ignore_case);
        let result_str: bool = Domain::is_valid_name_str(*name, *ignore_case);
        assert_eq!(result, result_str);
        assert_eq!(result, *expected);
    }
}

#[test]
fn is_valid_name_len() {
    let mut name: String = String::new();
    for i in 0..253 {
        if i % 50 == 0 && i != 0 {
            name.push('.');
        } else {
            name.push('a');
        }
    }
    assert_eq!(Domain::is_valid_name(name.as_bytes(), false), true);
    name.push('a');
    assert_eq!(Domain::is_valid_name(name.as_bytes(), false), false);
}
