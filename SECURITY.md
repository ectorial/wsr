# Security Policy

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Instead, report them privately by emailing:

**eduferreyraok@gmail.com**

Include as much detail as possible:
- Description of the vulnerability
- Steps to reproduce
- Potential impact

You will receive a response within 72 hours. Once the issue is confirmed and a fix is ready, a security advisory will be published and the fix will be released.

## Project status and execution design

The current checkout is a CLI/workspace scaffold with no working sandbox or workflow executor.
Do not use it as an established security boundary. The planned threat model, deny-by-default
capabilities, isolated system execution, and opt-in broad-access exception are described in
[docs/SECURITY-MODEL.md](docs/SECURITY-MODEL.md). Runtime/backend choices and several permission
contracts remain open. That design document is not a claim of implemented containment.
