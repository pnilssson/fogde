# Security policy

## Supported versions

Every push to `main` ships a new release, and Fogde updates itself. Only the latest release receives security fixes, so update before reporting.

| Version        | Supported |
| -------------- | --------- |
| Latest release | Yes       |
| Older releases | No        |

## Reporting a vulnerability

Report vulnerabilities privately through [GitHub's private vulnerability reporting](https://github.com/pnilssson/fogde/security/advisories/new). Please don't open a public issue, pull request or discussion.

Include what you can of:

- the Fogde version and operating system, and whether it runs on your machine or on a server
- steps to reproduce, or a proof of concept
- the impact, such as what an attacker can read, change or run

## What to expect

- An acknowledgement within 7 days.
- An assessment within 14 days, then updates until the issue is resolved.
- A fix in a new release, with a GitHub security advisory that credits you unless you prefer otherwise, and a CVE where warranted.

Please keep the details private until a fix is released or 90 days have passed, whichever comes first.

## Scope

In scope: the desktop app, the host, and the build, release and update pipeline in this repository.

Out of scope: vulnerabilities in the agent CLIs, `gh`, Herdr and other tools Fogde drives. Report those to their maintainers.
