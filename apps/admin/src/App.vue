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
    message.value = `Loaded ${payload.users.length} local users.`;
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
  <main class="app">
    <section class="shell">
      <header class="hero">
        <h1>WinPassage Admin</h1>
        <p>Manage local users on the central Windows Pro machine and perform audited administrator password resets.</p>
      </header>

      <section class="card">
        <h2>Connection</h2>
        <div class="grid two">
          <label>
            Server URL
            <input v-model="serverUrl" placeholder="http://CENTRAL-PC:4487" />
          </label>
          <label>
            Admin token
            <input v-model="adminToken" type="password" autocomplete="off" />
          </label>
        </div>
        <div class="actions">
          <button :disabled="loading || !adminToken" @click="loadUsers">Load users</button>
        </div>
        <p class="notice">The admin token is kept only in memory for this session.</p>
      </section>

      <p v-if="message" class="notice success">{{ message }}</p>
      <p v-if="error" class="notice error">{{ error }}</p>

      <section class="card">
        <h2>Local users</h2>
        <table class="table" v-if="users.length">
          <thead>
            <tr>
              <th>User</th>
              <th>Comment</th>
              <th>Status</th>
              <th>Password</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="user in users" :key="user.username" @click="selectedUser = user.username">
              <td><strong>{{ user.username }}</strong></td>
              <td>{{ user.full_name || '—' }}</td>
              <td>{{ user.disabled ? 'Disabled' : 'Enabled' }}</td>
              <td>{{ user.password_required ? 'Required' : 'Not required' }}</td>
            </tr>
          </tbody>
        </table>
        <p v-else class="notice">No users loaded yet.</p>
      </section>

      <section class="card">
        <h2>Reset password</h2>
        <p class="notice warning">Admin reset does not require the current password. Use only for owner-approved recovery and onboarding flows.</p>
        <div class="grid two">
          <label>
            Username
            <input v-model="selectedUser" placeholder="local user" />
          </label>
          <label>
            New password
            <input v-model="newPassword" type="password" autocomplete="new-password" />
          </label>
        </div>
        <label>
          Reason
          <textarea v-model="reason" rows="3" placeholder="Owner-approved reset, onboarding, emergency recovery..."></textarea>
        </label>
        <div class="actions">
          <button :disabled="loading || !canReset" @click="resetPassword">Reset password</button>
        </div>
      </section>
    </section>
  </main>
</template>
