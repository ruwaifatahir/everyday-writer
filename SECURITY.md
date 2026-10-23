# Security policy

## Reporting a vulnerability

Please report security problems privately through GitHub security advisories:
[Report a vulnerability](https://github.com/ruwaifatahir/everyday-writer/security/advisories/new).

Don't open a public issue for a security problem.

Include what you found, how to reproduce it and the impact you expect. You'll get a reply within 7 days.

## Scope

everyday-writer handles sensitive data: the text you select, your clipboard and your LLM API key. Of special interest:

- selected text, clipboard contents or API keys leaking into logs, files or anywhere other than the configured LLM provider
- selected text acting as instructions to the LLM (prompt injection) in a way that harms the user
- the clipboard not being restored after capture or replace
- API keys stored outside the Windows Credential Manager

## Supported versions

Only the latest release gets security fixes.
