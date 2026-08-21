# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in achat, please report it
**privately** via [GitHub Security Advisories](https://github.com/quadra-a/achat/security/advisories/new).

**Do not open a public issue for security vulnerabilities.**

## Scope

achat is a LAN-only tool with no authentication. It is designed for
trusted local networks. That said, we take the following reports seriously:

- Remote code execution
- Denial of service via crafted messages
- Path traversal in message storage
- mDNS spoofing leading to data exfiltration

## Response

We aim to acknowledge reports within 48 hours and provide a fix or
mitigation within 30 days.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |
