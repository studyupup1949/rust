//! Proc macros for addrezz.
//!
//! The [`addrezz!`] macro parses addresses at compile time and reports errors
//! at the call site. The name is deliberately distinct from the runtime
//! `addr!` declarative macro in `addrezz_core` to make it obvious at the
//! call site which flavour is in use.

use proc_macro::TokenStream;
use quote::quote;

use addrezz_core::{Addr, Host, ParseError, Scheme, Userinfo};

/// Parse an address at compile time.
///
/// Two input forms are accepted:
///
/// ```ignore
/// addrezz!("ssh://git@gitlab.com/racheedevel/addrezz");
/// addrezz! { git@gitlab.com:racheedevel/addrezz }
/// ```
///
/// **Note**: If the URL includes the scheme, only the quoted form will parse, as `scheme://` will
/// parse as a comment start before the macro could take effect.
#[proc_macro]
pub fn addrezz(input: TokenStream) -> TokenStream {
    let raw = extract_input(input);
    match Addr::parse(&raw) {
        Ok(a) => emit_addr(&a),
        Err(e) => emit_error(&raw, e),
    }
}

/// Parse multiple addresses. Accepts a comma- or semicolon-separated list.
/// Returns `[Addr; N]`.
#[proc_macro]
pub fn addrezz_vec(input: TokenStream) -> TokenStream {
    let mut out = Vec::new();
    for part in split_list(input.into()) {
        let cleaned = extract_from(part);
        if cleaned.is_empty() {
            continue;
        }
        match Addr::parse(&cleaned) {
            Ok(a) => out.push(emit_addr_expr(&a)),
            Err(e) => {
                let msg = format!("addrezz_vec!: failed to parse {cleaned:?}: {e}");
                return quote!(compile_error!(#msg)).into();
            }
        }
    }

    let n = out.len();
    let expanded = quote! {
        {
            let __arr: [::addrezz::__private::Addr; #n] = [ #( #out ),* ];
            __arr
        }
    };
    expanded.into()
}

fn extract_input(input: TokenStream) -> String {
    extract_from(input.into())
}

/// Take the address out of one input item, quoted or bare tokens.
fn extract_from(ts: proc_macro2::TokenStream) -> String {
    let trees: Vec<_> = ts.clone().into_iter().collect();
    if trees.len() == 1 {
        if let proc_macro2::TokenTree::Literal(lit) = &trees[0] {
            if let Ok(lit) = syn::parse_str::<syn::LitStr>(&lit.to_string()) {
                return lit.value();
            }
        }
    }
    clean_tokens(&ts.to_string())
}

/// Split a comma- or semicolon-separated list into its items.
fn split_list(ts: proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    let mut items = Vec::new();
    let mut current: Vec<proc_macro2::TokenTree> = Vec::new();
    for tt in ts {
        let is_separator = matches!(
            &tt,
            proc_macro2::TokenTree::Punct(p) if p.as_char() == ',' || p.as_char() == ';'
        );
        if is_separator {
            if !current.is_empty() {
                items.push(current.drain(..).collect());
            }
        } else {
            current.push(tt);
        }
    }
    if !current.is_empty() {
        items.push(current.into_iter().collect());
    }
    items
}

fn clean_tokens(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

fn emit_error(raw: &str, err: ParseError) -> TokenStream {
    let msg = format!("addrezz!: failed to parse {raw:?}: {err}");
    quote!(compile_error!(#msg)).into()
}

fn emit_addr(a: &Addr) -> TokenStream {
    let expr = emit_addr_expr(a);
    TokenStream::from(quote!(#expr))
}

fn emit_addr_expr(a: &Addr) -> proc_macro2::TokenStream {
    let scheme = emit_scheme(&a.scheme);
    let userinfo = emit_userinfo(a.userinfo.as_ref());
    let host = emit_host(&a.host);
    let port = match a.port {
        Some(p) => quote!(::core::option::Option::Some(#p)),
        None => quote!(::core::option::Option::None),
    };
    let path = &a.path;
    let query = match &a.query {
        Some(q) => quote!(::core::option::Option::Some(::std::string::String::from(#q))),
        None => quote!(::core::option::Option::None),
    };
    let fragment = match &a.fragment {
        Some(f) => quote!(::core::option::Option::Some(::std::string::String::from(#f))),
        None => quote!(::core::option::Option::None),
    };

    quote! {
        ::addrezz::__private::Addr {
            scheme: #scheme,
            userinfo: #userinfo,
            host: #host,
            port: #port,
            path: ::std::string::String::from(#path),
            query: #query,
            fragment: #fragment,
        }
    }
}

fn emit_scheme(s: &Scheme) -> proc_macro2::TokenStream {
    match s {
        Scheme::Other(o) => {
            quote!(::addrezz::__private::Scheme::Other(::std::string::String::from(#o)))
        }
        _ => {
            let ident = proc_macro2::Ident::new(
                scheme_variant_name(s),
                proc_macro2::Span::call_site(),
            );
            quote!(::addrezz::__private::Scheme::#ident)
        }
    }
}

fn scheme_variant_name(s: &Scheme) -> &'static str {
    match s {
        Scheme::Http => "Http",
        Scheme::Https => "Https",
        Scheme::Ws => "Ws",
        Scheme::Wss => "Wss",
        Scheme::Ssh => "Ssh",
        Scheme::Sftp => "Sftp",
        Scheme::Git => "Git",
        Scheme::GitSsh => "GitSsh",
        Scheme::GitHttps => "GitHttps",
        Scheme::GitHttp => "GitHttp",
        Scheme::Svn => "Svn",
        Scheme::SvnSsh => "SvnSsh",
        Scheme::Ftp => "Ftp",
        Scheme::Ftps => "Ftps",
        Scheme::File => "File",
        Scheme::Data => "Data",
        Scheme::Mailto => "Mailto",
        Scheme::Smtp => "Smtp",
        Scheme::Smtps => "Smtps",
        Scheme::Submission => "Submission",
        Scheme::Imap => "Imap",
        Scheme::Imaps => "Imaps",
        Scheme::Pop3 => "Pop3",
        Scheme::Pop3s => "Pop3s",
        Scheme::Ldap => "Ldap",
        Scheme::Ldaps => "Ldaps",
        Scheme::Postgres => "Postgres",
        Scheme::Mysql => "Mysql",
        Scheme::Mariadb => "Mariadb",
        Scheme::Mongodb => "Mongodb",
        Scheme::MongodbSrv => "MongodbSrv",
        Scheme::Redis => "Redis",
        Scheme::Rediss => "Rediss",
        Scheme::Clickhouse => "Clickhouse",
        Scheme::Cassandra => "Cassandra",
        Scheme::Sqlite => "Sqlite",
        Scheme::Amqp => "Amqp",
        Scheme::Amqps => "Amqps",
        Scheme::Mqtt => "Mqtt",
        Scheme::Mqtts => "Mqtts",
        Scheme::Nats => "Nats",
        Scheme::Kafka => "Kafka",
        Scheme::Grpc => "Grpc",
        Scheme::Grpcs => "Grpcs",
        Scheme::Sip => "Sip",
        Scheme::Sips => "Sips",
        Scheme::Tel => "Tel",
        Scheme::Xmpp => "Xmpp",
        Scheme::Irc => "Irc",
        Scheme::Ircs => "Ircs",
        Scheme::Coap => "Coap",
        Scheme::Coaps => "Coaps",
        Scheme::Stun => "Stun",
        Scheme::Stuns => "Stuns",
        Scheme::Turn => "Turn",
        Scheme::Turns => "Turns",
        Scheme::Dns => "Dns",
        Scheme::Ntp => "Ntp",
        Scheme::Other(_) => unreachable!(),
        _ => unreachable!("non_exhaustive Scheme variant"),
    }
}

fn emit_userinfo(ui: Option<&Userinfo>) -> proc_macro2::TokenStream {
    match ui {
        None => quote!(::core::option::Option::None),
        Some(u) => {
            let user = &u.username;
            let pass = match &u.password {
                Some(p) => quote!(::core::option::Option::Some(::std::string::String::from(#p))),
                None => quote!(::core::option::Option::None),
            };
            quote! {
                ::core::option::Option::Some(::addrezz::__private::Userinfo {
                    username: ::std::string::String::from(#user),
                    password: #pass,
                })
            }
        }
    }
}

fn emit_host(h: &Host) -> proc_macro2::TokenStream {
    match h {
        Host::Domain(d) => {
            quote!(::addrezz::__private::Host::Domain(::std::string::String::from(#d)))
        }
        Host::Ipv4(ip) => {
            let o = ip.octets();
            let (a, b, c, d) = (o[0], o[1], o[2], o[3]);
            quote! {
                ::addrezz::__private::Host::Ipv4(::std::net::Ipv4Addr::new(#a, #b, #c, #d))
            }
        }
        Host::Ipv6(ip) => {
            let s = ip.segments();
            let (s0, s1, s2, s3, s4, s5, s6, s7) =
                (s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]);
            quote! {
                ::addrezz::__private::Host::Ipv6(::std::net::Ipv6Addr::new(
                    #s0, #s1, #s2, #s3, #s4, #s5, #s6, #s7
                ))
            }
        }
    }
}
