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
