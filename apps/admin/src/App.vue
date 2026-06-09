<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ListUsersResponse, LocalUserSummary, PasswordChangeResponse } from './types';

const serverUrl = ref(localStorage.getItem('winpassage.admin.serverUrl') ?? 'http://localhost:4487');
const adminToken = ref('');
const users = ref<LocalUserSummary[]>([]);
const selectedUser = ref('');
const newPassword = ref('');
const reason = ref('');
const loading = ref(false);
const message = ref('');
const error = ref('');

const enabledUsers = computed(() => users.value.filter((user) => !user.disabled).length);
const disabledUsers = computed(() => users.value.filter((user) => user.disabled).length);
const passwordRequiredUsers = computed(() => users.value.filter((user) => user.password_required).length);
const selectedUserRecord = computed(() => users.value.find((user) => user.username === selectedUser.value) ?? null);
const passwordLengthHint = computed(() => `${newPassword.value.length}/12 minimum characters`);
const canReset = computed(() => selectedUser.value.length > 0 && newPassword.value.length >= 12 && adminToken.value.length > 0);

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

async function loadUsers() {
  loading.value = true;
  error.value = '';
  message.value = '';

  try {
    const payload = await request<ListUsersResponse>('/v1/users');
    users.value = payload.users;
    message.value = `Loaded ${payload.users.length} local users from the central machine.`;
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    loading.value = false;
  }
}

async function resetPassword() {
  loading.value = true;
  error.value = '';
  message.value = '';

  try {
    const payload = await request<PasswordChangeResponse>(`/v1/admin/users/${encodeURIComponent(selectedUser.value)}/password/reset`, {
      method: 'POST',
      body: JSON.stringify({
        new_password: newPassword.value,
        reason: reason.value || null,
        request_id: crypto.randomUUID(),
      }),
    });

    newPassword.value = '';
    reason.value = '';
    message.value = `${payload.message}. Request ${payload.request_id}`;
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    loading.value = false;
  }
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
          <span class="rail-item">Recovery</span>
          <span class="rail-item">Audit</span>
        </nav>

        <div class="rail-footer">
          <p class="metric-label">Session</p>
          <p class="muted">Admin token stays in memory only. Do not use this console on untrusted workstations.</p>
        </div>
      </aside>

      <section class="workspace">
        <header class="hero-panel">
          <div class="hero-grid">
            <div>
              <p class="eyebrow">Windows Pro central machine</p>
              <h1>Local user control without domain overhead.</h1>
              <p class="lead">
                Manage small-network Windows users, perform audited recovery resets, and keep password operations visible without turning the workstation into a domain controller.
              </p>
            </div>
            <div class="status-stack">
              <span class="status-pill success">Private network</span>
              <span class="status-pill warning">Admin only</span>
              <span class="status-pill">Audit-ready</span>
            </div>
          </div>
        </header>

        <section class="metric-grid" aria-label="User statistics">
          <article class="metric-card">
            <p class="metric-label">Loaded users</p>
            <p class="metric-value">{{ users.length }}</p>
            <p class="metric-hint">From the selected server</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Enabled</p>
            <p class="metric-value">{{ enabledUsers }}</p>
            <p class="metric-hint">Active local accounts</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Disabled</p>
            <p class="metric-value">{{ disabledUsers }}</p>
            <p class="metric-hint">Locked out of use</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Password required</p>
            <p class="metric-value">{{ passwordRequiredUsers }}</p>
            <p class="metric-hint">Policy-visible accounts</p>
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
            <button :disabled="loading || !adminToken" @click="loadUsers">Load users</button>
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
                <p class="muted">Select one local account before opening a recovery reset.</p>
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
                    <th>Password</th>
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
                        <span class="muted">Local account</span>
                      </span>
                    </td>
                    <td>{{ user.full_name || '—' }}</td>
                    <td>
                      <span class="badge" :class="user.disabled ? 'danger' : 'success'">{{ user.disabled ? 'Disabled' : 'Enabled' }}</span>
                    </td>
                    <td>
                      <span class="badge" :class="user.password_required ? 'success' : 'warning'">{{ user.password_required ? 'Required' : 'Not required' }}</span>
                    </td>
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

          <article class="danger-card">
            <div class="card-heading">
              <div>
                <h2>Recovery reset</h2>
                <p class="muted">Use only for owner-approved onboarding, recovery, or emergency workflows.</p>
              </div>
              <span class="status-pill danger">Privileged</span>
            </div>

            <p class="notice warning">Admin reset does not require the user's current password. The reason should be meaningful for audit review.</p>

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
                New password
                <input v-model="newPassword" type="password" autocomplete="new-password" />
              </label>
              <div class="password-meter" aria-label="Password length meter">
                <div class="meter-track">
                  <div class="meter-fill" :style="{ '--meter': `${Math.min(newPassword.length / 12 * 100, 100)}%` }"></div>
                </div>
                <p class="micro-copy">{{ passwordLengthHint }}</p>
              </div>
              <label>
                Reason
                <textarea v-model="reason" rows="3" placeholder="Owner-approved reset, onboarding, emergency recovery..."></textarea>
              </label>
            </div>

            <div class="form-footer">
              <p class="micro-copy">A request ID is generated and returned by the server.</p>
              <button class="danger" :disabled="loading || !canReset" @click="resetPassword">Reset password</button>
            </div>
          </article>
        </section>
      </section>
    </section>
  </main>
</template>
