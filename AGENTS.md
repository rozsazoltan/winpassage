# AGENTS.md

This file is the working map for AI agents and automation tools modifying WinPassage.

WinPassage is a Windows-first Rust/Tauri monorepo for small EU-oriented networks where a Windows 11 Pro machine is used as the central user and file machine instead of Active Directory or Windows Server. The product goal is controlled self-service password change, administrator recovery reset, mapped-drive credential refresh, and audit-friendly operation for small offices.

## Product intent

WinPassage should help small organizations reduce unsafe password handling without pretending to be a domain controller, compliance certificate, password vault, or enterprise identity platform.

Preserve these positioning rules:

- The central machine is a regular Windows Pro workstation used intentionally as a small-network hub.
- The app exists because some small companies do not want to operate Active Directory, Windows Server, Intune, or Entra ID for a very small office.
- The client app lets a regular user change only their own password by proving knowledge of the current password.
- The admin app supports owner-approved administrator password reset flows.
- The project can support NIS2-style cyber hygiene and audit processes, but must never claim to make an organization compliant by itself.
- The project must remain explicit about Windows Pro connection limits, licensing limits, and small-network scope.

## Repository map

```text
winpassage/
  README.md                         User-facing usage documentation.
  CONTRIBUTING.md                   Setup, service installation, local development, release process.
  SECURITY.md                       Responsible disclosure and security posture.
  AGENTS.md                         AI modification guide.

  crates/
    winpassage-core/                Shared domain rules: audit model, user model, password policy validation.
    winpassage-protocol/            API DTOs shared between service and apps.
    winpassage-windows/             Windows API wrappers for local users, mapped drives, credentials.
    winpassage-server/              HTTP API and Windows Service entrypoint.
    winpassage-agentctl/            Service install/start/stop helper CLI.

  apps/
    admin/                          Tauri 2 admin desktop app.
    client/                         Tauri 2 regular-user desktop app.

  .github/
    workflows/                      CI and release workflows.
    release/                        Verzly release configuration files.
```

## Architecture rules

Use these boundaries when adding or changing code.

### `winpassage-core`

Place platform-independent domain logic here:

- password policy validation;
- audit event structures;
- local user summary models;
- error categories that do not require Win32 types.

Do not add HTTP, Tauri, Windows API, or filesystem-specific service logic here.

### `winpassage-protocol`

Place API request and response DTOs here. These types are shared contracts. Keep them stable and explicit.

Good changes:

- adding a versioned request/response type;
- adding optional fields with safe defaults;
- adding structured error payloads.

Avoid:

- frontend-only state;
- Windows handles or platform-specific types;
- plaintext secret logging helpers.

### `winpassage-windows`

Place Windows-specific integration here:

- local user enumeration;
- local password reset/change wrappers;
- mapped drive reconnect logic;
- Credential Manager operations;
- service-control helper functions when they are reusable.

Prefer Win32 APIs over shelling out to `net user`, `net use`, or PowerShell. Do not pass passwords as command-line arguments.

### `winpassage-server`

The server is the authority for password changes on the central Windows machine.

It owns:

- HTTP API routing;
- admin token validation;
- self-service password change endpoint;
- admin password reset endpoint;
- audit logging;
- service mode and local config.

The server must not expose broad remote administration. Add endpoints narrowly and document their threat model.

### `winpassage-agentctl`

This CLI is for operator actions around the Windows Service:

- install;
- uninstall;
- start;
- stop;
- status.

Do not duplicate HTTP server behavior here.

### `apps/admin`

The admin app is a Tauri 2 UI for trusted operators. It may list users and trigger administrator password reset through the server API.

It must:

- visually separate normal actions from dangerous actions;
- display audit-oriented context such as request ID, reason, and selected user;
- keep admin secrets session-scoped unless a secure storage design is intentionally added;
- avoid storing passwords.

### `apps/client`

The client app is for regular users.

It may:

- submit username, current password, and new password to the self-service endpoint;
- reconnect configured mapped drives after the server confirms success;
- store non-secret settings such as server URL, username, and drive mappings.

It must not:

- call admin reset endpoints;
- let a user reset another user's password;
- store current or new passwords;
- reconnect drives until the password change is confirmed by the server.

## Security rules

These rules are mandatory.

- Never log plaintext passwords.
- Never put passwords in URLs, query strings, panic messages, debug output, telemetry, audit logs, or command-line arguments.
- Do not add a normal-client workflow that can reset another user's password.
- Do not make public-internet exposure sound safe. Recommend VPN, firewall allowlists, TLS/mTLS, or localhost proxying.
- Do not weaken password validation without a clearly documented reason.
- Do not silently ignore failed drive reconnect operations.
- Keep audit logs useful but secret-free.
- Treat admin reset as a privileged recovery operation, not a regular user flow.
- Prefer explicit error responses over ambiguous success messages.
- Avoid storing long-lived admin tokens in localStorage unless a secure-store implementation is added and documented.

If a requested change conflicts with these rules, implement the closest safe alternative and explain the tradeoff in the PR notes.

## UI and design direction

The desktop apps should feel modern, premium, and practical for a 2026 Windows business app.

Design principles:

- Use a command-center layout: rail/sidebar, workspace header, status cards, task panels.
- Keep dangerous actions visually isolated and clearly labeled.
- Use generous spacing, rounded panels, subtle gradients, soft borders, and high contrast text.
- Prefer calm professional visuals over playful styling.
- Make the app readable on compact Windows laptops and large desktop screens.
- Use progressive disclosure: show only necessary settings first, then advanced details.
- Make empty, loading, success, warning, and error states obvious.
- Do not introduce heavy UI frameworks without a clear reason.
- Keep the frontend dependency-light: Vue 3 + Vite + Tauri APIs should be enough for the base app.

Implementation notes:

- Shared visual language currently lives in each app's `src/styles.css`.
- Use semantic classes such as `app-shell`, `side-rail`, `workspace`, `hero-panel`, `metric-card`, `surface-card`, `danger-card`, `status-pill`, and `notice`.
- Do not use random decorative icons that obscure meaning. Labels must stand on their own.
- Keep form validation visible near the relevant action.

## Documentation rules

Follow the existing documentation split.

`README.md` should include:

- what the project is;
- who it is for;
- warnings and limitations;
- installation/usage overview;
- concrete usage examples;
- configuration reference;
- license section.

`README.md` should not include contributor-heavy development workflows.

`CONTRIBUTING.md` should include:

- local setup;
- required toolchain;
- service installation for development;
- test commands;
- release workflow;
- repo conventions.

Do not add a `LICENSE` item to the README menu. Keep the license section in the README body.

## Code style

Rust:

- Use typed error handling with `thiserror` where useful.
- Keep unsafe/FFI code isolated inside `winpassage-windows`.
- Prefer small functions with clear ownership boundaries.
- Validate input before calling Windows APIs.
- Keep secrets in memory only for the shortest practical time.
- Use `zeroize` or `secrecy` for newly added secret-bearing types when practical.

TypeScript/Vue:

- Use Composition API and explicit types.
- Keep API payload shapes aligned with `winpassage-protocol`.
- Store only non-secret preferences in `localStorage`.
- Use computed state for derived validation and UI status.
- Keep Tauri commands narrow and auditable.

Shell/automation:

- Scripts should be Windows-aware but not break Linux CI for non-Windows checks.
- Do not require global package installs when workspace scripts can be used.
- Keep generated files out of version control.

## Typical change patterns

### Add a new API field

1. Add the field to `winpassage-protocol`.
2. Update server handler validation.
3. Update admin/client app types.
4. Update UI copy only if the field is user-visible.
5. Add or adjust tests where possible.

### Add a new server endpoint

1. Decide whether it is admin-only or self-service.
2. Add DTOs to `winpassage-protocol`.
3. Add a narrow route in `winpassage-server`.
4. Add audit logging without secrets.
5. Update README usage only if operators/users need to call it.
6. Update `SECURITY.md` if the threat model changes.

### Change mapped drive behavior

1. Keep the server password-change success as the prerequisite.
2. Modify `winpassage-windows` drive code.
3. Update the Tauri client command wrapper.
4. Surface per-drive results in the UI.
5. Document Windows SMB credential limitations if behavior changes.

### Change password rules

1. Update `winpassage-core/src/password_policy.rs`.
2. Keep client-side validation consistent, but do not rely on it for enforcement.
3. Update README configuration/usage text if operators are affected.
4. Avoid weakening defaults.

## Testing expectations

Before finishing a change, run the relevant checks when the local environment supports them:

```bash
pnpm fmt:rust:check
pnpm lint:rust
pnpm test:rust
pnpm check:js
```

Windows-specific behavior should be verified on Windows 11 Pro when possible:

- local user listing;
- admin password reset;
- self-service password change;
- service install/start/stop;
- mapped drive reconnect;
- Credential Manager updates.

If a tool is unavailable in the current environment, state what was and was not verified.

## Commit and PR guidance

Use Conventional Commits:

```text
feat(client): add per-drive reconnect results
fix(server): reject empty password reset reasons
refactor(windows): isolate credential manager calls
docs(readme): clarify Windows Pro network limits
```

PR descriptions should mention:

- changed areas;
- user-visible behavior;
- security impact;
- tests performed;
- deployment notes, if any.

## Non-goals

Do not turn this repository into:

- an Active Directory clone;
- a domain controller;
- a password manager/vault;
- an RDP or remote administration suite;
- a cloud SaaS identity provider;
- a public internet password reset endpoint;
- a general-purpose Windows management agent.

Keep WinPassage focused: small-network Windows password self-service, admin recovery reset, mapped-drive refresh, and audit-friendly operation.


## Account lifecycle rules

AI agents must preserve the central architectural rule: WinPassage does not own user records. Windows local accounts are the source of truth. Application state may store configuration, audit events, replay protection metadata, and UI preferences only.

When changing account lifecycle behavior, keep these boundaries:

```text
crates/winpassage-windows/src/local_users.rs
  Owns local account enumeration, creation, deletion, enable/disable, password reset, and self-service password change wrappers.

crates/winpassage-windows/src/local_groups.rs
  Owns local Administrators membership checks and grant/revoke behavior. Resolve the built-in Administrators group through SID S-1-5-32-544 where possible instead of hard-coding a localized group name.

crates/winpassage-windows/src/sessions.rs
  Owns Windows session enumeration and logoff behavior.

crates/winpassage-protocol/src/lib.rs
  Owns API DTOs. Keep field names stable and snake_case for JSON compatibility.

crates/winpassage-server/src/http.rs
  Owns route wiring, request validation, audit event emission, and admin authorization.
```

Never add a database table or JSON file that mirrors the Windows user list. If a UI needs users, call `/v1/users`. If it needs sessions, call `/v1/sessions`.

Dangerous operations require all of the following:

```text
- admin authorization
- request_id support
- audit event with no passwords
- target username/session in the subject field
- meaningful error mapping
- guardrails for last-administrator removal or deletion
- UI confirmation for destructive operations
```

Password values must never be logged, returned, embedded in URLs, included in panic messages, or stored in application settings.

## Admin UI rules for lifecycle features

The admin UI should make real Windows-side effects visible. User deletion, administrator revocation, password reset, and session logoff must be visually separated from low-risk actions. Use warning/danger panels, exact username confirmation where destructive, and an audit reason field close to the action.

The table must communicate that data is loaded from Windows, not from a WinPassage database. Administrator status is a capability badge, not a localized Windows group label.
