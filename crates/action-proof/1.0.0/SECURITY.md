# Security Policy

## Supported Versions

Only the latest released version receives security fixes.

## Reporting

Please report security issues privately through GitHub Security Advisories for `wildmason/action-proof`.

## Scope

Security-sensitive bugs include:

- Passing a manifest that GitHub rejects.
- Skipping a configured failure without reporting it.
- Incorrectly treating dangerous `run` patterns as clean.
- Misclassifying remote `uses:` references as pinned.

`action-proof` reads local repository files and does not execute action steps.

