//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::infix::{Infix, InfixSignature};
use crate::context::SYSTEM_NS;

fn mksig(uri: &str, fst: &str, snd: &str, out: &str) -> InfixSignature {
    InfixSignature {
        uri: uri.to_owned(),
        first_arg: fst.to_owned(),
        second_arg: snd.to_owned(),
        output: out.to_owned(),
    }
}

// Build all the standard infix operators, covering basic mathematical
// and logical operators.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn standard_infix() -> Vec<Infix> {
    // equality
    let mut o = vec![Infix {
        operator: "==".to_owned(),
        allow_bags: true,
        commutative: true,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:string-equal",
                "string",
                "string",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:boolean-equal",
                "boolean",
                "boolean",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-equal",
                "integer",
                "integer",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:date-equal",
                "date",
                "date",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-equal",
                "double",
                "double",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:time-equal",
                "time",
                "time",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:dateTime-equal",
                "dateTime",
                "dateTime",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:3.0:function:dayTimeDuration-equal",
                "dayTimeDuration",
                "dayTimeDuration",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:3.0:function:yearMonthDuration-equal",
                "yearMonthDuration",
                "yearMonthDuration",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:anyURI-equal",
                "anyURI",
                "anyURI",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:x500Name-equal",
                "x500Name",
                "x500Name",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:rfc822Name-equal",
                "rfc822Name",
                "rfc822Name",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:hexBinary-equal",
                "hexBinary",
                "hexBinary",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:base64Binary-equal",
                "base64Binary",
                "base64Binary",
                "boolean",
            ),
        ],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: None,
    }];

    o.push(Infix {
        operator: "<".to_owned(),
        allow_bags: true,
        commutative: false,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-less-than",
                "integer",
                "integer",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-less-than",
                "double",
                "double",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:string-less-than",
                "string",
                "string",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:time-less-than",
                "time",
                "time",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:dateTime-less-than",
                "dateTime",
                "dateTime",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:date-less-than",
                "date",
                "date",
                "boolean",
            ),
        ],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: Some(">".to_owned()),
    });

    o.push(Infix {
        operator: ">=".to_owned(),
        allow_bags: true,
        commutative: false,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-greater-than-or-equal",
                "integer",
                "integer",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-greater-than-or-equal",
                "double",
                "double",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:string-greater-than-or-equal",
                "string",
                "string",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:time-greater-than-or-equal",
                "time",
                "time",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:dateTime-greater-than-or-equal",
                "dateTime",
                "dateTime",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:date-greater-than-or-equal",
                "date",
                "date",
                "boolean",
            ),
        ],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: Some("<=".to_owned()),
    });

    o.push(Infix {
        operator: "<=".to_owned(),
        allow_bags: true,
        commutative: false,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-less-than-or-equal",
                "integer",
                "integer",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-less-than-or-equal",
                "double",
                "double",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:string-less-than-or-equal",
                "string",
                "string",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:time-less-than-or-equal",
                "time",
                "time",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:dateTime-less-than-or-equal",
                "dateTime",
                "dateTime",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:date-less-than-or-equal",
                "date",
                "date",
                "boolean",
            ),
        ],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: Some(">=".to_owned()),
    });

    o.push(Infix {
        operator: ">".to_owned(),
        allow_bags: true,
        commutative: false,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-greater-than",
                "integer",
                "integer",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-greater-than",
                "double",
                "double",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:string-greater-than",
                "string",
                "string",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:time-greater-than",
                "time",
                "time",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:dateTime-greater-than",
                "dateTime",
                "dateTime",
                "boolean",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:date-greater-than",
                "date",
                "date",
                "boolean",
            ),
        ],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: Some("<".to_owned()),
    });

    o.push(Infix {
        operator: "&&".to_owned(),
        allow_bags: false,
        commutative: true,
        signatures: vec![mksig(
            "urn:oasis:names:tc:xacml:1.0:function:and",
            "boolean",
            "boolean",
            "boolean",
        )],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: None,
    });

    // 	infix comm (&&) = {

    o.push(Infix {
        operator: "||".to_owned(),
        allow_bags: false,
        commutative: true,
        signatures: vec![mksig(
            "urn:oasis:names:tc:xacml:1.0:function:or",
            "boolean",
            "boolean",
            "boolean",
        )],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: None,
    });

    o.push(Infix {
        operator: "+".to_owned(),
        allow_bags: false,
        commutative: true,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-add",
                "integer",
                "integer",
                "integer",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-add",
                "double",
                "double",
                "double",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:2.0:function:string-concatenate",
                "string",
                "string",
                "string",
            ),
        ],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: None,
    });

    o.push(Infix {
        operator: "-".to_owned(),
        allow_bags: false,
        commutative: false,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-subtract",
                "integer",
                "integer",
                "integer",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-subtract",
                "double",
                "double",
                "double",
            ),
        ],
        ns: vec![SYSTEM_NS.to_string()],
        inverse: None,
    });

    o.push(Infix {
        operator: "*".to_owned(),
        allow_bags: false,
        commutative: true,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-multiply",
                "integer",
                "integer",
                "integer",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-multiply",
                "double",
                "double",
                "double",
            ),
        ],

        ns: vec![SYSTEM_NS.to_string()],
        inverse: None,
    });

    o.push(Infix {
        operator: "/".to_owned(),
        allow_bags: false,
        commutative: false,
        signatures: vec![
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:integer-divide",
                "integer",
                "integer",
                "integer",
            ),
            mksig(
                "urn:oasis:names:tc:xacml:1.0:function:double-divide",
                "double",
                "double",
                "double",
            ),
        ],

        ns: vec![SYSTEM_NS.to_string()],
        inverse: None,
    });
    o
}
