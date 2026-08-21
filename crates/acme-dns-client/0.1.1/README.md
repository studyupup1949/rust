# acme-dns-client

[![Crates.io](https://img.shields.io/crates/v/acme-dns-client?color=blue
)](https://crates.io/crates/acme-dns-client)
[![Coverage](https://img.shields.io/badge/Coverage-Report-purple)](https://enigmacurry.github.io/rust-acme-dns-client/coverage/master/)

This Rust library implements the client API for
[joohoi/acme-dns](https://github.com/joohoi/acme-dns)

## Using acme-dns-client in Rust

The library is designed to be used directly from your Rust code. The basic
workflow is:

1. Create an `AcmeDnsClient` with your acme-dns API base URL.
2. Call `register()` once to obtain credentials (`username`, `password`,
   `subdomain`, `fulldomain`).
3. Persist those credentials somewhere safe (database, config file, etc).
4. For each DNS-01 challenge, call `update_txt()` with the stored credentials
   and the TXT token provided by your ACME client (e.g. Let’s Encrypt).

### Simple example

```rust
use acme_dns_client::{AcmeDnsClient, Credentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Construct a client pointing at your acme-dns instance
    let client = AcmeDnsClient::new("https://auth.acme-dns.io")?;

    // 2. One-time registration (optionally with allowfrom CIDRs)
    let creds: Credentials = client.register(None).await?;
    println!("Registered acme-dns credentials: {creds:#?}");

    // You should now:
    // - Store `creds` somewhere persistent.
    // - Create a CNAME:
    //   _acme-challenge.yourdomain.com -> creds.fulldomain

    // 3. Later, when your ACME library asks you to present a DNS-01 token:
    let dns01_token = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ"; // replace with real token
    client.update_txt(&creds, dns01_token).await?;

    Ok(())
}
```

### Using environment variables

The crate also provides helpers to construct both the client and credentials
from environment variables:

```rust
use acme_dns_client::{AcmeDnsClient, Credentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ACME_DNS_API_BASE must be set, e.g. "https://auth.acme-dns.io"
    let client = AcmeDnsClient::from_env()?;

    // Credentials are expected in:
    //   ACME_DNS_USERNAME
    //   ACME_DNS_PASSWORD
    //   ACME_DNS_SUBDOMAIN
    //   ACME_DNS_FULLDOMAIN
    //   ACME_DNS_ALLOWFROM (optional, comma-separated)
    let creds = Credentials::from_env()?;

    let dns01_token = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ"; // replace with real token
    client.update_txt(&creds, dns01_token).await?;

    Ok(())
}
```

## Testing with manual command line interaction

This library is intended to be used programmatically in your own Rust
code. However, there is also included a simple manual command line
tool for testing the interaction with ACME-DNS:

```bash
## Example manual test CLI command:
CMD="cargo run --features cli --bin acme-dns-cli -- "

## Make sure to fill in your own values for each exported variable below:

export ACME_DNS_API_BASE="https://auth.acme-dns.io"

# Health check
${CMD} health

# One-time registration sets env vars in your current shell:
eval "$(${CMD} register | jq -r '
  [
    "export ACME_DNS_USERNAME="  + (.username  | @sh),
    "export ACME_DNS_PASSWORD="  + (.password  | @sh),
    "export ACME_DNS_SUBDOMAIN=" + (.subdomain | @sh),
    "export ACME_DNS_FULLDOMAIN=" + (.fulldomain | @sh),
    "export ACME_DNS_ALLOWFROM=" + ((.allowfrom | join(",")) | @sh)
  ]
  | join("\n")
')"


# Update the TXT record using your account set in the environment:
# (This token should comes from your ACME API provider (e.g., Let's Encrypt), 
#  For testing purposes you can use this fake token that is exactly 43 chars long)
${CMD} update --txt "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ"

# Verify the update matches the TXT record you updated:
dig +short TXT "$ACME_DNS_FULLDOMAIN"
```
