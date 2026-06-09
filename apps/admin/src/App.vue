<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ActionResponse, ListSessionsResponse, ListUsersResponse, LocalSessionSummary, LocalUserSummary } from './types';

const serverUrl = ref(localStorage.getItem('winpassage.admin.serverUrl') ?? 'http://localhost:4487');
const adminToken = ref('');
const users = ref<LocalUserSummary[]>([]);
const sessions = ref<LocalSessionSummary[]>([]);
const selectedUser = ref('');
const newPassword = ref('');
const reason = ref('');
const loading = ref(false);
const message = ref('');
const error = ref('');

const createUsername = ref('');
const createFullName = ref('');
const createPassword = ref('');
const createMustChange = ref(true);
const createEnabled = ref(true);
const createAdmin = ref(false);

const deleteConfirmation = ref('');
const deleteLogoffSessions = ref(true);
const deleteProfile = ref(false);

const enabledUsers = computed(() => users.value.filter((user) => !user.disabled).length);
const disabledUsers = computed(() => users.value.filter((user) => user.disabled).length);
const adminUsers = computed(() => users.value.filter((user) => user.is_administrator).length);
const activeSessions = computed(() => sessions.value.filter((session) => session.username).length);
const selectedUserRecord = computed(() => users.value.find((user) => user.username === selectedUser.value) ?? null);
const selectedUserSessions = computed(() => sessions.value.filter((session) => session.username?.toLowerCase() === selectedUser.value.toLowerCase()));
const passwordLengthHint = computed(() => `${newPassword.value.length}/12 minimum characters`);
const canReset = computed(() => selectedUser.value.length > 0 && newPassword.value.length >= 12 && adminToken.value.length > 0);
const canCreate = computed(() => createUsername.value.length > 0 && createPassword.value.length >= 12 && adminToken.value.length > 0);
const canDelete = computed(() => selectedUser.value.length > 0 && deleteConfirmation.value === selectedUser.value && adminToken.value.length > 0);

function persistSettings() {
  localStorage.setItem('winpassage.admin.serverUrl', serverUrl.value.replace(/\/$/, ''));
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  persistSettings();
  const response = await fetch(`${serverUrl.value.replace(/\/$/, '')}${path}`, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${adminToken.value}`,
      ...(init?.headers ?? {}),
    },
  });

  const payload = await response.json().catch(() => ({}));

  if (!response.ok) {
    throw new Error(payload.error ?? `Request failed with ${response.status}`);
  }

  return payload as T;
}

async function runTask(task: () => Promise<string>, refresh = true) {
  loading.value = true;
  error.value = '';
  message.value = '';

  try {
    message.value = await task();
    if (refresh) {
      await Promise.allSettled([loadUsers(false), loadSessions(false)]);
    }
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    loading.value = false;
  }
}

async function loadUsers(showMessage = true) {
  const payload = await request<ListUsersResponse>('/v1/users');
  users.value = payload.users;
  if (showMessage) {
    message.value = `Loaded ${payload.users.length} local Windows accounts.`;
  }
}

async function loadSessions(showMessage = true) {
  const payload = await request<ListSessionsResponse>('/v1/sessions');
  sessions.value = payload.sessions;
  if (showMessage) {
    message.value = `Loaded ${payload.sessions.length} Windows sessions.`;
  }
}

async function refreshAll() {
  await runTask(async () => {
    const [userPayload, sessionPayload] = await Promise.all([
      request<ListUsersResponse>('/v1/users'),
      request<ListSessionsResponse>('/v1/sessions'),
    ]);
    users.value = userPayload.users;
    sessions.value = sessionPayload.sessions;
    return `Loaded ${userPayload.users.length} users and ${sessionPayload.sessions.length} sessions.`;
  }, false);
}

async function createUser() {
  await runTask(async () => {
    const payload = await request<ActionResponse>('/v1/users', {
      method: 'POST',
      body: JSON.stringify({
        username: createUsername.value,
        full_name: createFullName.value || null,
        password: createPassword.value,
        must_change_password: createMustChange.value,
        enabled: createEnabled.value,
        admin: createAdmin.value,
        reason: reason.value || 'Admin-created local account',
        request_id: crypto.randomUUID(),
      }),
    });

    selectedUser.value = createUsername.value;
    createUsername.value = '';
    createFullName.value = '';
    createPassword.value = '';
    createAdmin.value = false;
    return `${payload.message}. Request ${payload.request_id}`;
  });
}

async function resetPassword() {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/users/${encodeURIComponent(selectedUser.value)}/password/reset`, {
      method: 'POST',
      body: JSON.stringify({
        new_password: newPassword.value,
        reason: reason.value || null,
        request_id: crypto.randomUUID(),
      }),
    });

    newPassword.value = '';
    return `${payload.message}. Request ${payload.request_id}`;
  });
}

async function setEnabled(enabled: boolean) {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/users/${encodeURIComponent(selectedUser.value)}/enabled`, {
      method: 'POST',
      body: JSON.stringify({
        enabled,
        reason: reason.value || null,
        request_id: crypto.randomUUID(),
      }),
    });
    return `${payload.message}. Request ${payload.request_id}`;
  });
}

async function setAdministrator(enabled: boolean) {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/users/${encodeURIComponent(selectedUser.value)}/admin`, {
      method: 'POST',
      body: JSON.stringify({
        enabled,
        reason: reason.value || null,
        request_id: crypto.randomUUID(),
      }),
    });
    return `${payload.message}. Request ${payload.request_id}`;
  });
}

async function logoffSession(sessionId: number) {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/sessions/${sessionId}/logoff`, {
      method: 'POST',
      body: JSON.stringify({
        reason: reason.value || 'Admin requested session logoff',
        request_id: crypto.randomUUID(),
      }),
    });
    return `${payload.message}. Request ${payload.request_id}`;
  });
}

async function deleteUser() {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/users/${encodeURIComponent(selectedUser.value)}`, {
      method: 'DELETE',
      body: JSON.stringify({
        delete_profile: deleteProfile.value,
        logoff_sessions: deleteLogoffSessions.value,
        reason: reason.value || 'Admin deleted local account',
        request_id: crypto.randomUUID(),
      }),
    });

    selectedUser.value = '';
    deleteConfirmation.value = '';
    deleteProfile.value = false;
    return `${payload.message}. Request ${payload.request_id}`;
  });
}
</script>

<template>
  <main class="app app-admin">
    <section class="app-shell">
      <aside class="side-rail" aria-label="WinPassage Admin navigation">
        <div class="brand-block">
          <div class="brand-mark">WP</div>
          <div>
            <p class="brand-title">WinPassage</p>
            <p class="brand-subtitle">Admin console</p>
          </div>
        </div>

        <nav class="rail-menu" aria-label="Workspace sections">
          <span class="rail-item active"><span class="rail-dot"></span>Overview</span>
          <span class="rail-item">Users</span>
          <span class="rail-item">Access</span>
          <span class="rail-item">Sessions</span>
          <span class="rail-item">Audit</span>
        </nav>

        <div class="rail-footer">
          <p class="metric-label">Security note</p>
          <p class="muted">This console changes real local Windows accounts. Use only from trusted admin workstations.</p>
        </div>
      </aside>

      <section class="workspace">
        <header class="hero-panel">
          <div class="hero-grid">
            <div>
              <p class="eyebrow">Windows Pro central machine</p>
              <h1>Local Windows user management without domain overhead.</h1>
              <p class="lead">
                Read the current local accounts directly from Windows, create or remove users, manage administrator access, and log off active sessions with audit-friendly operator reasons.
              </p>
            </div>
            <div class="status-stack">
              <span class="status-pill success">Windows source of truth</span>
              <span class="status-pill warning">Privileged actions</span>
              <span class="status-pill">EU small networks</span>
            </div>
          </div>
        </header>

        <section class="metric-grid" aria-label="User statistics">
          <article class="metric-card">
            <p class="metric-label">Loaded users</p>
            <p class="metric-value">{{ users.length }}</p>
            <p class="metric-hint">Current Windows accounts</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Enabled</p>
            <p class="metric-value">{{ enabledUsers }}</p>
            <p class="metric-hint">Allowed to sign in</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Administrators</p>
            <p class="metric-value">{{ adminUsers }}</p>
            <p class="metric-hint">Local admin group</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Sessions</p>
            <p class="metric-value">{{ activeSessions }}</p>
            <p class="metric-hint">Named Windows sessions</p>
          </article>
        </section>

        <section class="surface-card">
          <div class="card-heading">
            <div>
              <h2>Connection</h2>
              <p class="muted">Connect to the WinPassage service on the central Windows Pro machine.</p>
            </div>
            <span class="status-pill" :class="adminToken ? 'success' : 'warning'">{{ adminToken ? 'Token provided' : 'Token required' }}</span>
          </div>
          <div class="grid two">
            <label>
              Server URL
              <input v-model="serverUrl" placeholder="http://CENTRAL-PC:4487" />
            </label>
            <label>
              Admin token
              <input v-model="adminToken" type="password" autocomplete="off" placeholder="Session-only operator token" />
            </label>
          </div>
          <div class="actions">
            <button :disabled="loading || !adminToken" @click="refreshAll">Load users & sessions</button>
            <button class="secondary" :disabled="loading" @click="persistSettings">Save server URL</button>
          </div>
        </section>

        <p v-if="message" class="notice success">{{ message }}</p>
        <p v-if="error" class="notice error">{{ error }}</p>

        <section class="panel-grid">
          <article class="surface-card">
            <div class="card-heading">
              <div>
                <h2>Local users</h2>
                <p class="muted">The list is loaded from the central machine's local Windows accounts, not from a WinPassage database.</p>
              </div>
              <span class="status-pill">{{ selectedUser || 'No selection' }}</span>
            </div>

            <div v-if="users.length" class="table-wrap">
              <table class="table">
                <thead>
                  <tr>
                    <th>User</th>
                    <th>Full name</th>
                    <th>Status</th>
                    <th>Access</th>
                    <th>Sessions</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="user in users"
                    :key="user.username"
                    :class="{ active: selectedUser === user.username }"
                    @click="selectedUser = user.username"
                  >
                    <td>
                      <span class="user-cell">
                        <strong>{{ user.username }}</strong>
                        <span class="muted">Local Windows account</span>
                      </span>
                    </td>
                    <td>{{ user.full_name || '—' }}</td>
                    <td>
                      <span class="badge" :class="user.disabled ? 'danger' : 'success'">{{ user.disabled ? 'Disabled' : 'Enabled' }}</span>
                    </td>
                    <td>
                      <span class="badge" :class="user.is_administrator ? 'warning' : 'neutral'">{{ user.is_administrator ? 'Administrator' : 'Standard' }}</span>
                    </td>
                    <td>{{ user.active_session_count }}</td>
                  </tr>
                </tbody>
              </table>
            </div>

            <div v-else class="empty-state">
              <div>
                <h3>No users loaded</h3>
                <p>Enter the admin token, then load local users from the central machine.</p>
              </div>
            </div>
          </article>

          <article class="surface-card">
            <div class="card-heading">
              <div>
                <h2>Create local account</h2>
                <p class="muted">Creates a real Windows local user on the central machine.</p>
              </div>
              <span class="status-pill warning">Admin action</span>
            </div>

            <div class="grid two">
              <label>
                Username
                <input v-model="createUsername" placeholder="julia" />
              </label>
              <label>
                Full name
                <input v-model="createFullName" placeholder="Julia Nagy" />
              </label>
              <label>
                Initial password
                <input v-model="createPassword" type="password" autocomplete="new-password" />
              </label>
              <label>
                Audit reason
                <input v-model="reason" placeholder="New employee onboarding" />
              </label>
            </div>
            <div class="toggle-grid">
              <label class="toggle-row"><input v-model="createMustChange" type="checkbox" /> Require password change at next sign-in</label>
              <label class="toggle-row"><input v-model="createEnabled" type="checkbox" /> Enable account immediately</label>
              <label class="toggle-row"><input v-model="createAdmin" type="checkbox" /> Add to local Administrators group</label>
            </div>
            <div class="actions">
              <button :disabled="loading || !canCreate" @click="createUser">Create user</button>
            </div>
          </article>
        </section>

        <section class="panel-grid">
          <article class="danger-card">
            <div class="card-heading">
              <div>
                <h2>Account control</h2>
                <p class="muted">Change status, reset password, and manage administrator access for the selected user.</p>
              </div>
              <span class="status-pill danger">Privileged</span>
            </div>

            <p class="notice warning">Every operation below changes the Windows account directly and should include an audit reason.</p>

            <div v-if="selectedUserRecord" class="selected-card">
              <p class="metric-label">Selected account</p>
              <strong>{{ selectedUserRecord.username }}</strong>
              <span class="muted">{{ selectedUserRecord.full_name || 'No full name set' }}</span>
            </div>

            <div class="grid">
              <label>
                Username
                <input v-model="selectedUser" placeholder="local user" />
              </label>
              <label>
                New password for recovery reset
                <input v-model="newPassword" type="password" autocomplete="new-password" />
              </label>
              <div class="password-meter" aria-label="Password length meter">
                <span :style="{ width: `${Math.min(100, (newPassword.length / 12) * 100)}%` }"></span>
              </div>
              <p class="micro-copy">{{ passwordLengthHint }}</p>
              <label>
                Audit reason
                <textarea v-model="reason" rows="3" placeholder="Owner-approved recovery reset, onboarding, role change, offboarding..."></textarea>
              </label>
            </div>

            <div class="actions wrap">
              <button class="danger" :disabled="loading || !canReset" @click="resetPassword">Reset password</button>
              <button class="secondary" :disabled="loading || !selectedUser" @click="setEnabled(false)">Disable</button>
              <button class="secondary" :disabled="loading || !selectedUser" @click="setEnabled(true)">Enable</button>
              <button class="secondary" :disabled="loading || !selectedUser" @click="setAdministrator(true)">Grant admin</button>
              <button class="danger" :disabled="loading || !selectedUser" @click="setAdministrator(false)">Revoke admin</button>
            </div>
          </article>

          <article class="danger-card">
            <div class="card-heading">
              <div>
                <h2>Delete account</h2>
                <p class="muted">Requires typing the exact username before the account is removed from Windows.</p>
              </div>
              <span class="status-pill danger">Destructive</span>
            </div>

            <p class="notice error">Deleting a user removes the local Windows account. Profile deletion removes the Windows profile data for the selected local account. Keep this disabled unless offboarding requires profile cleanup.</p>
            <div class="grid">
              <label>
                Type selected username to confirm
                <input v-model="deleteConfirmation" :placeholder="selectedUser || 'select a user first'" />
              </label>
              <label class="toggle-row"><input v-model="deleteLogoffSessions" type="checkbox" /> Log off matching active sessions first</label>
              <label class="toggle-row"><input v-model="deleteProfile" type="checkbox" /> Request profile deletion after account removal</label>
            </div>
            <div class="actions">
              <button class="danger" :disabled="loading || !canDelete" @click="deleteUser">Delete selected user</button>
            </div>
          </article>
        </section>

        <section class="surface-card">
          <div class="card-heading">
            <div>
              <h2>Windows sessions</h2>
              <p class="muted">Use logoff for stale or offboarding sessions before deleting or disabling an account.</p>
            </div>
            <span class="status-pill">{{ sessions.length }} sessions</span>
          </div>

          <div v-if="sessions.length" class="table-wrap">
            <table class="table">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>User</th>
                  <th>State</th>
                  <th>Client</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="session in sessions" :key="session.session_id">
                  <td>{{ session.session_id }}</td>
                  <td>{{ session.domain ? `${session.domain}\\${session.username || '—'}` : session.username || '—' }}</td>
                  <td><span class="badge neutral">{{ session.state }}</span></td>
                  <td>{{ session.client_name || (session.is_console ? 'Console' : '—') }}</td>
                  <td><button class="secondary small" :disabled="loading || !session.username" @click="logoffSession(session.session_id)">Log off</button></td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-else class="empty-state">
            <div>
              <h3>No sessions loaded</h3>
              <p>Load sessions from the central machine to manage active sign-ins.</p>
            </div>
          </div>
        </section>
      </section>
    </section>
  </main>
</template>
