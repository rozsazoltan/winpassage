# WinPassage

WinPassage is an EU-oriented Windows password self-service toolkit for small offices and local networks where a regular Windows 11 Pro machine is used as the central file and user machine instead of running Active Directory or Windows Server.

It provides a Windows background agent, an admin desktop app, and a client desktop app for controlled local-user password changes, mapped drive credential refresh, and audit-friendly password change workflows.

> [!WARNING]
> WinPassage is not a compliance certificate, not an Active Directory replacement, and not a way to bypass Microsoft licensing. It is a small-network helper for environments where Windows Pro is intentionally used as a central machine. Validate the final deployment with your legal, licensing, and security requirements.

- [What it solves](#what-it-solves)
  - [Windows Pro central machine](#windows-pro-central-machine)
  - [EU-oriented small network workflow](#eu-oriented-small-network-workflow)
  - [What it is not](#what-it-is-not)
- [Get started](#get-started)
  - [Quick install](#quick-install)
  - [Server machine](#server-machine)
  - [Multiple central machines](#multiple-central-machines)
  - [Admin app](#admin-app)
  - [Client app](#client-app)
- [Usage](#usage)
  - [Account lifecycle management](#account-lifecycle-management)
  - [Admin password reset](#admin-password-reset)
  - [Self-service password change](#self-service-password-change)
  - [Mapped drives](#mapped-drives)
  - [Audit log](#audit-log)
  - [Updates](#updates)
  - [Release rollback](#release-rollback)
  - [Connection limits](#connection-limits)
  - [Security model](#security-model)
- [Configuration](#configuration)
- [Known Issues](#known-issues)
- [Contributing](#contributing)

Read on to understand the intended network model. Or jump straight to [Get started](#get-started) if you already have a Windows Pro central machine and want to install the agent.

## What it solves

Small companies often start with a single Windows 11 Pro workstation that stores shared folders and local user accounts. That can be enough for a small internal network, but it creates operational problems:

- users cannot safely change their own central-machine password without admin help;
- mapped drives keep using old credentials after a password change;
- password resets are hard to audit consistently;
- password handling often happens through chat, paper notes, or direct admin intervention;
- introducing Active Directory, Windows Server, Intune, or Entra ID may be disproportionate for a very small office.

WinPassage adds a controlled password-change layer around that central Windows Pro machine.

### Windows Pro central machine

The central machine runs `winpassage-server.exe` as a Windows Service. It exposes a restricted local-network API for:

- listing local Windows users for admins;
- creating, disabling, deleting, and managing local Windows users;
- granting and revoking local administrator access;
- listing and logging off Windows sessions for offboarding;
- resetting local user passwords from the admin app;
- allowing users to change only their own password by providing the current password;
- writing audit events without logging secrets.

The admin and client apps are Tauri 2 desktop apps. They are intentionally separate:

- `WinPassageAdmin` is for the owner, IT operator, or trusted administrator;
- `WinPassageClient` is for regular users who only need self-service password change.

### EU-oriented small network workflow

WinPassage is designed for EU small-business environments that need more disciplined password handling without adding a full domain infrastructure.

It helps implement practical controls around:

- self-service password changes;
- password policy validation before submission;
- least-privilege client behavior;
- local audit trails;
- no cloud dependency;
- no stored plaintext passwords;
- mapped drive credential refresh after a successful password change.

> [!IMPORTANT]
> NIS2 and similar audit frameworks are organization-level obligations. WinPassage can support access-control and cyber-hygiene processes, but it cannot make an organization compliant by itself. For the official EU text, see [Directive (EU) 2022/2555](https://eur-lex.europa.eu/eli/dir/2022/2555/oj/eng).

### What it is not

WinPassage is not:

- Active Directory;
- Windows Server;
- an LDAP or Kerberos implementation;
- a remote desktop product;
- a password vault;
- a public-internet admin panel;
- a replacement for MFA, backups, endpoint protection, or documented security policies.

If the environment grows beyond a small local network, move to Windows Server, Active Directory, Entra ID, Intune, a NAS with proper identity management, or another managed identity platform.

## Get started

WinPassage is used in three places:

```text
central Windows Pro machine  ->  winpassage-server.exe Windows Service
administrator workstation    ->  WinPassageAdmin
user workstation             ->  WinPassageClient
```

Detailed deployment and service setup belongs in `CONTRIBUTING.md`. This README focuses on daily usage and operational behavior.

> [!IMPORTANT]
> Run the server agent only on trusted private networks. Do not expose the API directly to the public internet. Use VPN, firewall allowlists, and TLS/mTLS in production.

### Quick install

Download and install **WinPassageAdmin** on the Windows Pro machine that should become the central computer. You do not need to manually copy the server executables next to the app.

Open **WinPassageAdmin** with **Run as administrator**, then choose:

```text
Overview → Server on this computer → Install on this computer
```

The admin app downloads these official release assets from GitHub, verifies their matching `.sha256` files, copies them to `C:\Program Files\WinPassage`, installs the Windows Service, and starts it:

```text
winpassage-server.exe
winpassage-agentctl.exe
winpassage-updater.exe
```

Only these update sources are accepted:

```text
https://github.com/rozsazoltan/winpassage
https://api.github.com/repos/rozsazoltan/winpassage/releases/*
```

The server listens on the configured IP:port. Clients and additional admin profiles connect with:

```text
http://SERVER-IP:PORT
```

Example:

```text
http://192.168.1.10:4487
```

### Server machine

The central machine is the Windows 11 Pro computer that owns the local user accounts and shared folders. A machine becomes a WinPassage server only after the server service is installed from **WinPassageAdmin** with administrator privileges.

Use **Settings → Advanced → Remove local server service** in WinPassageAdmin to stop and remove the local WinPassage service. This removes only WinPassage service components and optionally the copied WinPassage executables. It never deletes Windows users or profiles.

Before users start using WinPassage, the operator should confirm:

```text
- the WinPassage service is running
- the API is reachable from the private network
- the admin token is configured
- the audit log path is writable
- the Windows firewall allows the selected private-network port
- backups for shared data exist
```

Health check:

```powershell
Invoke-RestMethod http://CENTRAL-PC:4487/health
```

Expected response:

```json
{
  "status": "ok",
  "service": "winpassage-server"
}
```

### Multiple central machines

A single `WinPassageAdmin` installation can keep local shortcuts for multiple standalone WinPassage servers. This is useful when one operator manages several small networks, branches, workshops, or office rooms where each location has its own Windows Pro central machine.

Each server profile contains only connection metadata:

```text
Display name
Network or location name
IP address or DNS name
Port
Protocol
Operator notes
```

The profile does not store Windows users and should not store admin passwords. User lists, sessions, administrator status, and account changes are always loaded from the selected Windows machine at request time.

> [!IMPORTANT]
> Treat each server as a separate authority. Switching the active profile changes which Windows machine receives user-management actions. Verify the selected server before creating, deleting, disabling, or resetting users.

### Admin app

Install `WinPassageAdmin` on the central machine or an administrator workstation.

On the central machine, the app requires a Windows administrator account for local server installation, demotion, and service management. Standard users see a locked screen with instructions to switch to an administrator account.

On first launch, complete the short setup screen before the in-app guide starts. Then register one or more server profiles:

```text
Name:        Office server
Network:     Budapest office
IP/DNS:      192.168.1.10
Port:        4487
Protocol:    http or https
Admin token: the value from WINPASSAGE_ADMIN_TOKEN
```

Use **Load users & sessions** to list users and sessions from the selected server. Select a user, type a new password, add a reason, and run **Reset password**. The active server is shown in the dashboard because every operation changes the selected Windows machine directly.

> [!WARNING]
> Admin reset does not require the user's current password. Use it only for administrator-controlled recovery, onboarding, and emergency flows. Regular users should use the client app.

### Client app

Install `WinPassageClient` on each workstation that maps shared drives from the central machine.

On first launch, complete the short setup screen before the in-app guide starts. The server address is configured during deployment or support. It is shown to the user, but its modification is kept behind **Advanced connection settings** so regular users do not accidentally point the app at the wrong machine.

For normal use, the user enters:

```text
Username
Current password
New password
Confirm new password
```

After a successful password change, the client can reconnect configured mapped drives with the new credentials.

WinPassageClient starts with **zero** default drive mappings. Add only the drive letters and share paths used on that workstation. Each drive can be mounted, unmounted, edited, or deleted individually.

## Usage

### Account lifecycle management

WinPassage reads the current local Windows accounts from the selected central machine. It does not keep a separate user database and does not try to mirror Windows users into application storage.

The admin console is designed to support these local Windows account operations:

```text
- list current local Windows users
- create a local user
- delete a local user
- enable or disable a local user
- reset a user's password as an administrator
- grant or revoke local administrator access
- list active Windows sessions
- log off a selected session before offboarding or account deletion
```

> [!WARNING]
> User creation, deletion, administrator grants, administrator revokes, and session logoff are privileged Windows operations. Use a dedicated test account first and keep the service reachable only from a trusted private network, VPN, or mTLS-protected admin channel.

> [!IMPORTANT]
> The local Administrators group is resolved through the built-in Administrators SID where possible. The UI should display administrator access as a capability, not as a localized group name.

### Admin password reset

Use the admin app when the administrator needs to set a new password for a local user without knowing the old one.

API equivalent:

```powershell
$body = @{
  new_password = "NewStrongPassword123!"
  reason = "Owner-approved reset"
  request_id = [guid]::NewGuid().ToString()
} | ConvertTo-Json

Invoke-RestMethod `
  -Method Post `
  -Uri "http://CENTRAL-PC:4487/v1/users/julia/password/reset" `
  -Headers @{ Authorization = "Bearer replace-with-a-long-random-token" } `
  -ContentType "application/json" `
  -Body $body
```

### Self-service password change

Use the client app for normal user-driven password changes. The server verifies the current password through Windows and only changes that user's own password.

API equivalent:

```powershell
$body = @{
  username = "julia"
  current_password = "OldPassword123!"
  new_password = "NewStrongPassword123!"
  request_id = [guid]::NewGuid().ToString()
} | ConvertTo-Json

Invoke-RestMethod `
  -Method Post `
  -Uri "http://CENTRAL-PC:4487/v1/me/password/change" `
  -ContentType "application/json" `
  -Body $body
```

### Mapped drives

After a successful password change, the client app can reconnect configured mapped drives using the new credentials.

The current implementation uses Windows network-drive APIs instead of passing passwords through `net use` command-line arguments.

Typical flow:

```text
1. user changes central password
2. server returns success
3. client reconnects configured drives with username + new password
4. individual drives can also be mounted or unmounted manually
5. Windows updates remembered mappings for the user profile
```

> [!NOTE]
> Windows may reject multiple simultaneous SMB sessions to the same server using different credentials. Close open Explorer windows and applications using the share before reconnecting drives.

### Audit log

The server writes JSON Lines audit events to:

```text
C:\ProgramData\WinPassage\audit.jsonl
```

Each event records:

```text
timestamp
event name
actor
subject user
request id
source IP
result
reason
```

Passwords are never written to the audit log.

Example:

```json
{"timestamp":"2026-06-09T10:30:00Z","event":"self_password_change","actor":"julia","subject":"julia","result":"success","request_id":"..."}
```

### Updates

`winpassage-updater.exe` is the dedicated updater entrypoint. It must only use the official repository:

```text
https://github.com/rozsazoltan/winpassage
```

Do not configure custom update hosts or mirrors. Update application should verify artifact names, checksums, and the official GitHub source before replacing installed binaries.

### Release rollback

Use the manual **Delete Release** workflow only when a release must be withdrawn, for example after publishing assets with incorrect names or broken binaries.

GitHub Actions input:

```text
version: 0.2.0
confirm: DELETE 0.2.0
```

The workflow deletes the GitHub Release and the matching `vX.Y.Z` tag. It does not change already installed machines. Publish a corrected patch release for normal users whenever possible.

> [!WARNING]
> Release deletion is destructive. Prefer a new patch release if the incorrect release may already have been downloaded.

### Connection limits

Windows 11 Pro can work as a small central file machine, but it remains a client operating system. [Microsoft Windows 11 use terms](https://www.microsoft.com/content/dam/microsoft/usetm/documents/windows/11/oem-%28pre-installed%29/UseTerms_OEM_Windows_11_English.pdf) allow up to 20 other devices to access software on the licensed device for features such as file services, print services, IIS, Internet connection sharing, and telephony services.

For practical deployments, treat WinPassage as suitable for roughly the same scale:

| Environment | Recommendation |
|---|---|
| 1-5 users | Good fit for a simple local network. |
| 6-15 users | Good fit if network shares are modest and backups are disciplined. |
| 16-20 users | Works only if the Windows Pro connection limit and performance are acceptable. |
| 20+ users | Use Windows Server, AD/Entra ID, NAS identity management, or a proper domain setup. |

> [!WARNING]
> The 20-device limit is a Windows licensing and platform boundary, not a WinPassage feature flag. Do not design around it with multiplexing or connection pooling.

### Security model

WinPassage follows these rules:

- regular users can only request a password change for themselves;
- self-service change requires the current password;
- admin reset requires an admin bearer token;
- passwords are accepted only in JSON request bodies, never URL query strings;
- passwords are not logged;
- mapped drive reconnect happens only after server-confirmed password change;
- generated audit records avoid secrets;
- the API is intended for private LAN or VPN use;
- admin server profiles are connection shortcuts, not an application user database;
- client server address changes are intentionally hidden behind an advanced confirmation flow.

Recommended production controls:

```text
- private network profile only
- inbound firewall allowlist
- VPN or overlay network
- TLS/mTLS in front of the agent
- long random admin token
- regular audit log review
- endpoint protection on the central machine
- offline backups of shared data
```

## Configuration

The server reads configuration from environment variables.

| Variable | Default | Description |
|---|---|---|
| `WINPASSAGE_BIND` | `127.0.0.1:4487` | Address and port for the server API. |
| `WINPASSAGE_ADMIN_TOKEN` | empty | Required for admin endpoints. |
| `WINPASSAGE_REQUIRE_TLS` | `true` | Refuses non-TLS deployment when set to `true`. |
| `WINPASSAGE_AUDIT_LOG` | `C:\ProgramData\WinPassage\audit.jsonl` | Audit log output path on Windows. |
| `WINPASSAGE_PASSWORD_MIN_LENGTH` | `12` | Minimum accepted password length. |

> [!TIP]
> Use Windows system environment variables for the service account context. Restart the service after changing them.

Admin and client app settings are local workstation settings:

| Setting | Stored by | Secret? | Notes |
|---|---|---:|---|
| Server profiles | Admin app | No | Display name, network name, IP/DNS, port, protocol, notes. |
| Active server | Admin app | No | Determines which standalone Windows machine receives admin actions. |
| Server URL | Client app | No | Hidden behind advanced connection settings to avoid accidental changes. |
| Username and drive paths | Client app | No | Stored only for user convenience. |
| Admin token | Admin app | Yes | Session-only by default; do not store without secure storage. |
| Passwords | Neither app | Yes | Never stored. |

## Known Issues

### Windows Pro is not Windows Server

A Windows 11 Pro central machine is practical for very small networks, but it is not built for larger file-service workloads. For more than 20 accessing devices, use a server-grade setup.

### Server profiles are local shortcuts

The admin app can remember multiple server profiles, but those profiles do not create federation, synchronization, replication, or shared identity between servers. Each Windows Pro machine remains independent.

### Local accounts only

WinPassage targets local Windows accounts on the central machine. Microsoft accounts, domain accounts, and Entra ID identities are outside the first implementation scope.

### Password policy comes from Windows

WinPassage performs early validation, but Windows remains the authority. Password history, complexity, and local security policy may still reject a password.

### Shares may keep stale sessions

Mapped drives can remain open in Explorer or other applications. If reconnect fails, close applications using the share and retry.

### Compliance depends on deployment

WinPassage helps with audit-friendly password change workflows, but NIS2, ISO 27001, and similar frameworks require wider organizational, technical, and procedural controls.

## Contributing

See: `CONTRIBUTING.md`.


## License & Acknowledgments

WinPassage is released under the GNU Affero General Public License v3.0 only.

Created by Zoltán Rózsa.

### WinPassage update and install policy

WinPassageAdmin installs the local server components by downloading the official `winpassage-server`, `winpassage-agentctl`, and `winpassage-updater` release assets from `https://github.com/rozsazoltan/winpassage`. The app verifies the matching `.sha256` file before copying an executable into `C:\Program Files\WinPassage`. Custom update hosts, mirrors, and alternate repositories are intentionally unsupported.

`winpassage-agentctl` is the small privileged service-control helper. It installs, starts, stops, and removes the Windows Service. `winpassage-updater` is the dedicated boot/update boundary: it should check official releases, verify assets, apply service binary updates, and then leave the WinPassage service running. WinPassageAdmin does not need to start with Windows.

The user interface must stay concise for auditors and security operators: keep install and update actions as buttons, move detailed inputs into modals, avoid crowded dashboards, and keep About clear about what WinPassage is and is not.

