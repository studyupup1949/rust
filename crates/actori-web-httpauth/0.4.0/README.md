# actori-web-httpauth

[![Latest Version](https://img.shields.io/crates/v/actori-web-httpauth.svg)](https://crates.io/crates/actori-web-httpauth)
[![Latest Version](https://docs.rs/actori-web-httpauth/badge.svg)](https://docs.rs/actori-web-httpauth)
[![dependency status](https://deps.rs/crate/actori-web-httpauth/0.4.0/status.svg)](https://deps.rs/crate/actori-web-httpauth/0.4.0)
![Build Status](https://github.com/actori/actori-web-httpauth/workflows/CI/badge.svg?branch=master&event=push)
![Apache 2.0 OR MIT licensed](https://img.shields.io/badge/license-Apache2.0%2FMIT-blue.svg)

HTTP authentication schemes for [actori-web](https://github.com/actori/actori-web) framework.

Provides:
 * typed [Authorization] and  [WWW-Authenticate] headers
 * [extractors] for an [Authorization] header
 * [middleware] for easier authorization checking

All supported schemas are actori [Extractors](https://docs.rs/actori-web/1.0.0/actori_web/trait.FromRequest.html),
and can be used both in the middlewares and request handlers.

## Supported schemes

 * [Basic](https://tools.ietf.org/html/rfc7617)
 * [Bearer](https://tools.ietf.org/html/rfc6750)
