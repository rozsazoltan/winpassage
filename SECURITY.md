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
- password values in URL query strings, command-line arguments, or logs;
- silent or accidental client-side server switching by regular users;
- treating admin server profiles as centralized identity inventory.

## Reporting vulnerabilities

Report vulnerabilities privately through the repository owner's preferred private channel. Do not open a public issue with exploit details, secrets, or deployment-specific credentials.

## Security expectations

A secure WinPassage deployment should use firewall allowlists, VPN or overlay networking, TLS/mTLS in front of the agent, endpoint protection, offline backups, and regular audit review.


## Local administrator access

WinPassage can grant or revoke membership in the local Administrators group. Treat this as a high-impact administrative action. The server must refuse to remove or delete the final local administrator account, and the UI must require explicit confirmation before destructive account lifecycle operations.

## Windows sessions

Session logoff can interrupt active work. Use it only for owner-approved offboarding, stale sessions, or recovery workflows. Audit entries must include the target session and the operator reason, but never any password value.


## Multiple standalone servers

Admin server profiles are convenience shortcuts for trusted operators. They should contain only non-secret connection metadata and should not store Windows users or plaintext admin credentials. Each selected server remains an independent Windows authority.

Before running privileged actions, the UI must make the active server visible enough that an operator can confirm the correct network and IP address.

## Client connection settings

The client app should display the configured server address but keep editing behind an advanced confirmation flow. This reduces accidental misconfiguration for regular users and prevents support issues where password changes are sent to the wrong central machine.

## Update security

WinPassage updates must be sourced only from `https://github.com/rozsazoltan/winpassage` release assets and the matching GitHub releases API. The updater must reject custom mirrors, custom repositories, non-HTTPS URLs, and release asset URLs outside the official repository.

Privileged server updates should be applied through the dedicated updater/service-control boundary, not directly by the user-facing Tauri apps.

## Local server installation safety

WinPassage Admin must require a Windows administrator account for local service installation, service removal, and server demotion. Opening the admin app from a standard account should show a locked state with instructions to switch to an administrator account.

WinPassage Client must not contain server installation or service-management features.

Changing the service port changes how clients and admins connect to the server. Operators must distribute the correct `IP:port` value to client profiles after changing the port.
