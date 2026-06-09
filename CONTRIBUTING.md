# Contributing

Contributor and operator setup notes for the WinPassage monorepo.

- [Setup](#setup)
- [Workspace rules](#workspace-rules)
- [Local development](#local-development)
- [Windows service setup](#windows-service-setup)
- [Local network configuration](#local-network-configuration)
- [Multi-server admin profiles](#multi-server-admin-profiles)
- [Testing](#testing)
- [GitHub Actions](#github-actions)
- [Release workflow](#release-workflow)
- [Documentation](#documentation)

## Setup

Install the local toolchain:

```bash
mise trust
mise install
corepack enable
corepack prepare pnpm@10.32.1 --activate
pnpm install
```

The repository assumes Rust stable, Node.js 22, pnpm 10, and the Verzly release toolchain in local development or GitHub Actions.

## Workspace rules

The repository is source-first. Generated output belongs under `.cache`, `dist`, app build folders, or package-manager stores.

Use these boundaries:

- `apps/admin`: Tauri 2 admin UI.
- `apps/client`: Tauri 2 end-user UI.
- `crates/winpassage-core`: shared validation and domain logic.
- `crates/winpassage-protocol`: request/response DTOs shared by apps and service.
- `crates/winpassage-windows`: Windows API wrappers.
- `crates/winpassage-server`: HTTP API and Windows Service entrypoint.
- `crates/winpassage-agentctl`: service install/start/stop helper.

Do not put Windows password logic in the frontend. The server service owns local-user password changes. The client app only handles self-service UI and mapped drive reconnect after a successful server response.

The admin app may store local server profiles for multiple standalone WinPassage servers. These profiles are connection shortcuts only; they must not duplicate Windows users or become a second source of truth.

## Local development

Root scripts:

```bash
pnpm check:rust
pnpm check:js
pnpm build:server
pnpm build:agentctl
pnpm build:admin
pnpm build:client
```

Run the server locally:

```bash
WINPASSAGE_BIND=127.0.0.1:4487 \
WINPASSAGE_ADMIN_TOKEN=dev-token \
WINPASSAGE_REQUIRE_TLS=false \
cargo run -p winpassage-server -- serve
```

Run the admin app:

```bash
pnpm dev:admin
```

Run the client app:

```bash
pnpm dev:client
```

## Windows service setup

Build release binaries on Windows:

```powershell
cargo build --release -p winpassage-server -p winpassage-agentctl
```

Copy them to:

```text
C:\Program Files\WinPassage\
```

Install the service:

```powershell
& "C:\Program Files\WinPassage\winpassage-agentctl.exe" install `
  --server-bin "C:\Program Files\WinPassage\winpassage-server.exe"
```

Start, stop, and remove it:

```powershell
& "C:\Program Files\WinPassage\winpassage-agentctl.exe" start
& "C:\Program Files\WinPassage\winpassage-agentctl.exe" stop
& "C:\Program Files\WinPassage\winpassage-agentctl.exe" uninstall
```


## Local network configuration

Set the machine-level environment variables before starting the service:

```powershell
setx WINPASSAGE_BIND "0.0.0.0:4487" /M
setx WINPASSAGE_ADMIN_TOKEN "replace-with-a-long-random-token" /M
setx WINPASSAGE_REQUIRE_TLS "false" /M
setx WINPASSAGE_AUDIT_LOG "C:\ProgramData\WinPassage\audit.jsonl" /M
```

`WINPASSAGE_REQUIRE_TLS=false` is acceptable only for isolated private LAN, VPN, or first setup testing. Production deployments should use TLS/mTLS in front of the agent or keep the agent bound to loopback behind a trusted local proxy.

Allow the private-network port:

```powershell
New-NetFirewallRule `
  -DisplayName "WinPassage Agent" `
  -Direction Inbound `
  -Action Allow `
  -Protocol TCP `
  -LocalPort 4487 `
  -Profile Private
```

Check the agent after start:

```powershell
Invoke-RestMethod http://localhost:4487/health
```

## Multi-server admin profiles

Use the admin app's server registry when one operator manages multiple independent Windows Pro central machines. Add one profile per machine:

```text
Name: Office server
Network: Budapest office
IP/DNS: 192.168.1.10
Port: 4487
Protocol: http or https
```

Development expectations:

- selecting a profile must update the active server URL;
- switching profiles should clear loaded users and sessions;
- profiles must not store Windows users, passwords, or long-lived admin tokens;
- the selected profile must be visible before privileged operations;
- client server settings must stay behind an advanced confirmation flow.

## Testing

Run touched-area checks before opening a pull request:

```bash
pnpm fmt:rust:check
pnpm lint:rust
pnpm test:rust
pnpm check:js
```

Windows-specific code should be tested on Windows 11 Pro because local user, service control, and mapped drive APIs cannot be fully validated on Linux CI.

## GitHub Actions

Use the published Verzly action line:

- `verzly/rust-cache@v0`
- `verzly/cargo-release@v0`
- `verzly/github-release@v0`
- `verzly/tauri-release@v0`

Only update major action refs when the matching distribution repository has the new major tag.

## Release workflow

The release workflow is Windows-only for app artifacts.

Release configuration lives under:

```text
.github/release/winpassage.github-release.toml
.github/release/winpassage-server.cargo-release.toml
.github/release/winpassage-agentctl.cargo-release.toml
.github/release/winpassage-admin.tauri-release.toml
.github/release/winpassage-client.tauri-release.toml
```

The intended flow:

```text
1. prepare release branch and version files
2. run Rust checks
3. run JS/Tauri checks
4. build server and agentctl exe artifacts
5. build admin and client installers
6. upload artifacts to GitHub Release
7. merge release branch back to master
```

## Documentation

The root `README.md` is user-facing and should stay focused on usage, deployment, warnings, and operational behavior.

Development, setup, testing, and release instructions belong in this file.

Do not add a README to every crate unless the crate is published or consumed separately.


## Testing account lifecycle operations

Use a disposable local Windows account before testing destructive operations.

```powershell
net user winpassage-test "OldPassword123!" /add
```

Run the server locally with an admin token:

```powershell
$env:WINPASSAGE_BIND = "127.0.0.1:4487"
$env:WINPASSAGE_ADMIN_TOKEN = "dev-token"
$env:WINPASSAGE_REQUIRE_TLS = "false"
cargo run -p winpassage-server -- serve
```

Useful API smoke tests:

```powershell
Invoke-RestMethod `
  -Uri "http://127.0.0.1:4487/v1/users" `
  -Headers @{ Authorization = "Bearer dev-token" }

Invoke-RestMethod `
  -Uri "http://127.0.0.1:4487/v1/sessions" `
  -Headers @{ Authorization = "Bearer dev-token" }
```

Delete the disposable user after testing:

```powershell
net user winpassage-test /delete
Get-CimInstance Win32_UserProfile |
  Where-Object { $_.LocalPath -eq "C:\Users\winpassage-test" } |
  Remove-CimInstance
```

Do not test account deletion, administrator revocation, or session logoff on your only administrator account.
