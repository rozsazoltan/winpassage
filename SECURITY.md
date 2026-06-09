# Security Policy

WinPassage handles local Windows account password changes. Treat every deployment as a privileged administration surface.

## Supported deployment model

Supported first-party model:

- private LAN or VPN only;
- Windows 11 Pro central machine;
- local Windows accounts only;
- server agent running as a Windows Service;
- admin token stored as a machine-level environment variable;
- audit log stored outside user-writable project directories.

Unsupported model:

- public internet exposure;
- unauthenticated API access;
- shared admin tokens in chat or documents;
- normal users calling admin reset endpoints;
- password values in URL query strings, command-line arguments, or logs.

## Reporting vulnerabilities

Report vulnerabilities privately through the repository owner's preferred private channel. Do not open a public issue with exploit details, secrets, or deployment-specific credentials.

## Security expectations

A secure WinPassage deployment should use firewall allowlists, VPN or overlay networking, TLS/mTLS in front of the agent, endpoint protection, offline backups, and regular audit review.


## Local administrator access

WinPassage can grant or revoke membership in the local Administrators group. Treat this as a high-impact administrative action. The server must refuse to remove or delete the final local administrator account, and the UI must require explicit confirmation before destructive account lifecycle operations.

## Windows sessions

Session logoff can interrupt active work. Use it only for owner-approved offboarding, stale sessions, or recovery workflows. Audit entries must include the target session and the operator reason, but never any password value.

## Profile deletion

Account deletion and profile deletion are separate high-impact actions. Profile deletion uses Windows profile APIs and should be used only after active sessions are logged off and the operator has confirmed that local profile data is no longer required.
