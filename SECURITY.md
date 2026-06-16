# Security Policy

We take the security of MochiFlow seriously. Thank you for helping keep the
project and its users safe.

## Supported versions

MochiFlow is pre-1.x in stabilization; security fixes are applied to the latest
released version on the `main` branch. Please make sure you can reproduce an
issue against the latest version before reporting.

| Version | Supported |
| --- | --- |
| latest (`main`) | ✅ |
| older releases | ❌ |

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues,
pull requests, or discussions.**

Instead, report them privately through GitHub's
[private vulnerability reporting](https://github.com/ELUNOX/mochiflow/security/advisories/new)
(Security tab → "Report a vulnerability"). If you cannot use that channel,
contact the maintainer [@ELUNOX](https://github.com/ELUNOX).

Please include, as much as you can:

- the type of issue and the component affected (CLI, engine, contracts, adapters);
- the version / commit you tested against and your environment (OS, Rust toolchain);
- step-by-step instructions to reproduce;
- proof-of-concept or exploit code, if available;
- the impact, including how an attacker might exploit it.

## What to expect

- We aim to acknowledge a report within a few business days.
- We will investigate, keep you informed of progress, and agree on a
  coordinated disclosure timeline.
- We will credit you for the discovery once a fix is released, unless you prefer
  to remain anonymous.

## Scope notes

MochiFlow is a workflow + living-spec engine and a CLI; it invokes the project's
own verify commands and delegates `git push` / PR creation to external tools
(`gh` / a custom `pr_driver`). Vulnerabilities in those external tools should be
reported to their respective maintainers, but issues in how MochiFlow invokes or
configures them are in scope.
