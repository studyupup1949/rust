//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::attribute::Attribute;
use crate::context::SYSTEM_NS;

fn mk_attr(id: &str, typedef: &str, cat: &str, uri: &str) -> Attribute {
    Attribute {
        id: id.to_owned(),
        typedef: typedef.to_owned(),
        category: cat.to_owned(),
        uri: uri.to_owned(),
        ns: vec![SYSTEM_NS.to_string()],
    }
}

// build all the "standard" attributes
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn standard_attributes() -> Vec<Attribute> {
    // 10.2.5 Attributes
    let mut a = vec![mk_attr(
        "currentTime",
        "time",
        "environmentCat",
        "urn:oasis:names:tc:xacml:1.0:environment:current-time",
    )];
    a.push(mk_attr(
        "currentDate",
        "time",
        "environmentCat",
        "urn:oasis:names:tc:xacml:1.0:environment:current-date",
    ));
    a.push(mk_attr(
        "currentDateTime",
        "time",
        "environmentCat",
        "urn:oasis:names:tc:xacml:1.0:environment:current-dateTime",
    ));
    // 10.2.6 Identifiers
    a.push(mk_attr(
        "subjectLocalityDnsName",
        "string",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:authn-locality:dns-name",
    ));
    a.push(mk_attr(
        "subjectLocalityIpAddress",
        "string",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:authn-locality:ip-address",
    ));
    a.push(mk_attr(
        "authenticationMethod",
        "string",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:authentication-method",
    ));
    a.push(mk_attr(
        "authenticationTime",
        "dateTime",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:authentication-time",
    ));
    a.push(mk_attr(
        "keyInfo",
        "string",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:key-info",
    ));
    a.push(mk_attr(
        "requestTime",
        "dateTime",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:request-time",
    ));
    a.push(mk_attr(
        "sessionStartTime",
        "dateTime",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:session-start-time",
    ));
    a.push(mk_attr(
        "subjectId",
        "string",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:subject-id",
    ));
    a.push(mk_attr(
        "subjectIdQualifier",
        "string",
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject:subject-id-qualifier",
    ));
    a.push(mk_attr(
        "resourceLocation",
        "string",
        "resourceCat",
        "urn:oasis:names:tc:xacml:1.0:resource:resource-location",
    ));
    a.push(mk_attr(
        "resourceId",
        "string",
        "resourceCat",
        "urn:oasis:names:tc:xacml:1.0:resource:resource-id",
    ));
    a.push(mk_attr(
        "simpleFileName",
        "string",
        "resourceCat",
        "urn:oasis:names:tc:xacml:1.0:resource:simple-file-name",
    ));
    a.push(mk_attr(
        "actionId",
        "string",
        "actionCat",
        "urn:oasis:names:tc:xacml:1.0:action:action-id",
    ));
    a.push(mk_attr(
        "impliedAction",
        "string",
        "actionCat",
        "urn:oasis:names:tc:xacml:1.0:action:implied-action",
    ));

    a
}
