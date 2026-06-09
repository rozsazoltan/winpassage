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
  - [Server machine](#server-machine)
  - [Admin app](#admin-app)
  - [Client app](#client-app)
- [Usage](#usage)
  - [Admin password reset](#admin-password-reset)
  - [Self-service password change](#self-service-password-change)
  - [Mapped drives](#mapped-drives)
  - [Audit log](#audit-log)
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
- resetting local user passwords from the admin app;
- allowing users to change only their own password by providing the current password;
- writing audit events without logging secrets.

The admin and client apps are Tauri 2 desktop apps. They are intentionally separate:

- `WinPassage Admin` is for the owner, IT operator, or trusted administrator;
- `WinPassage Client` is for regular users who only need self-service password change.

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
administrator workstation    ->  WinPassage Admin
user workstation             ->  WinPassage Client
```

Detailed deployment and service setup belongs in `CONTRIBUTING.md`. This README focuses on daily usage and operational behavior.

> [!IMPORTANT]
> Run the server agent only on trusted private networks. Do not expose the API directly to the public internet. Use VPN, firewall allowlists, and TLS/mTLS in production.

### Server machine

The central machine is the Windows 11 Pro computer that owns the local user accounts and shared folders.

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

### Admin app

Install `WinPassage Admin` on the central machine or an administrator workstation.

Open the app and set:

```text
Server URL:  http://CENTRAL-PC:4487
Admin token: the value from WINPASSAGE_ADMIN_TOKEN
```

Use **Load users** to list local users on the central machine. Select a user, type a new password, add a reason, and run **Reset password**.

> [!WARNING]
> Admin reset does not require the user's current password. Use it only for administrator-controlled recovery, onboarding, and emergency flows. Regular users should use the client app.

### Client app

Install `WinPassage Client` on each workstation that maps shared drives from the central machine.

The user enters:

```text
Server URL
Username
Current password
New password
Confirm new password
```

After a successful password change, the client can reconnect configured mapped drives with the new credentials.

Default drive examples:

```text
S: -> \\CENTRAL-PC\Shared
I: -> \\CENTRAL-PC\Internal
G: -> \\CENTRAL-PC\Groups
```

## Usage

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
  -Uri "http://CENTRAL-PC:4487/v1/admin/users/julia/password/reset" `
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

After a successful password change, the client app reconnects selected mapped drives using the new credentials.

The current implementation uses Windows network-drive APIs instead of passing passwords through `net use` command-line arguments.

Typical flow:

```text
1. user changes central password
2. server returns success
3. client disconnects configured drives
4. client reconnects them with username + new password
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
- the API is intended for private LAN or VPN use.

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

## Known Issues

### Windows Pro is not Windows Server

A Windows 11 Pro central machine is practical for very small networks, but it is not built for larger file-service workloads. For more than 20 accessing devices, use a server-grade setup.

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
