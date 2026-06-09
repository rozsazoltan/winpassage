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
aube install
```

The repository assumes Rust stable, Node.js 24, aube, and the Verzly release toolchain in local development or GitHub Actions.

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
aube run check:rust
aube run check:js
aube run build:server
aube run build:agentctl
aube run build:admin
aube run build:client
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
aube run dev:admin
```

Run the client app:

```bash
aube run dev:client
```

## Windows service setup

Build release binaries on Windows:

```powershell
cargo build --release -p winpassage-server -p winpassage-agentctl -p winpassage-updater
```

The preferred operator flow is to place the three service executables next to WinPassage Admin and use the admin app's **Make this computer a WinPassage server** button.

Manual service install remains available for validation:

```powershell
& "C:\Program Files\WinPassage\winpassage-agentctl.exe" install `
  --server-bin "C:\Program Files\WinPassage\winpassage-server.exe" `
  --bind "0.0.0.0:4487" `
  --admin-token "replace-with-a-long-random-token" `
  --require-tls false
```

Start, stop, and remove it:

```powershell
& "C:\Program Files\WinPassage\winpassage-agentctl.exe" start
& "C:\Program Files\WinPassage\winpassage-agentctl.exe" stop
& "C:\Program Files\WinPassage\winpassage-agentctl.exe" uninstall
```

## Local network configuration

The admin install flow writes the selected bind address and admin token into the service command line. Environment variables are still supported for foreground development runs:

```powershell
$env:WINPASSAGE_BIND = "0.0.0.0:4487"
$env:WINPASSAGE_ADMIN_TOKEN = "replace-with-a-long-random-token"
$env:WINPASSAGE_REQUIRE_TLS = "false"
$env:WINPASSAGE_AUDIT_LOG = "C:\ProgramData\WinPassage\audit.jsonl"
```


## CI cache behavior

Rust Quality uses a single `rust-cache run` wrapper around the full strict Rust command sequence. The workflow runs on `windows-latest`, so the wrapper must execute the cargo sequence through PowerShell/`pwsh`, not `bash -lc`. Calling `bash` from inside `rust-cache run` can resolve to WSL on GitHub-hosted Windows runners and fail when no WSL distribution is installed.

Keep the `.cache` target directory shared across the check, clippy, test, doctest, and rustdoc phases inside the same job. Do not split the Rust quality workflow back into several independent `rust-cache run` steps unless the cache toolchain contract changes or there is measured evidence that separate steps are faster.

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

The release workflow invokes release builders with their current documented CLI shape:

```bash
cargo-release build --config .github/release/winpassage-server.cargo-release.toml
tauri-release build --config .github/release/winpassage-admin.tauri-release.toml
```

Do not pass `--output`, `--windows`, or `--verbose` to `tauri-release build`. Output directories, platform enablement, commands, and artifact globs belong in the `.tauri-release.toml` files.


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


## Release workflow compatibility

The release workflow uses `verzly/github-release@latest` for `prepare`, `finalize`, and failed release branch cleanup. The release branch commit is verified with `git rev-parse HEAD` instead of a tool-specific helper subcommand so the workflow stays compatible with the published `github-release` CLI contract. Release assets must be passed to `github-release finalize` with `--assets`.

## Updater source policy

`winpassage-updater` is the dedicated update entrypoint. It must only resolve metadata and release assets from `https://github.com/rozsazoltan/winpassage` and the matching GitHub releases API. Do not add custom update hosts, mirrors, or user-configurable repositories.

## Server installation from WinPassage Admin

WinPassage Client must stay client-only. Do not add service installation, server demotion, or local machine management features to the client app.

The server installation flow belongs to WinPassage Admin only:

```text
WinPassage Admin -> copy service binaries -> install Windows Service -> start service
```

The admin app requires an elevated Windows administrator account before showing the normal console. If it is opened from a standard account, it must show the lock screen and instructions to switch to an administrator account.

The install form should collect:

```text
- source folder containing winpassage-server.exe, winpassage-agentctl.exe, winpassage-updater.exe
- install folder, normally C:\Program Files\WinPassage
- bind host, normally 0.0.0.0 for private LAN use
- port, for example 4487
- long admin token
- TLS boundary setting
```

The demotion flow removes only the WinPassage service and optionally the copied WinPassage executables. It must not delete Windows users, Windows profiles, shares, mapped drives, or audit logs.
