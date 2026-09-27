# Security Policy

## Reporting a vulnerability

Please avoid publishing exploitable security issues before they can be fixed. Open a minimal GitHub issue asking for a private reporting channel, without including secrets or weaponized details.

## Threat model

Lumina launches third-party Windows software inside a compatibility environment. The project therefore treats downloaded executables and runtime archives as untrusted inputs until their source and integrity can be validated.

Lumina should:

- never run downloaded files through a shell string;
- keep launcher state in user-writable application directories;
- avoid elevated privileges;
- verify downloads when an authoritative checksum/signature is available;
- clearly show the source of external downloads;
- keep logs free of credentials and session tokens.

## Out of scope

Lumina does not attempt to bypass, disable, tamper with or emulate anti-cheat systems.
