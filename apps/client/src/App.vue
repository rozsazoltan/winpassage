<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, reactive, ref } from 'vue';
import type { DriveMapping, PasswordChangeResponse, ReconnectMappedDrivesResponse } from './types';

const serverUrl = ref(localStorage.getItem('winpassage.client.serverUrl') ?? 'http://CENTRAL-PC:4487');
const username = ref(localStorage.getItem('winpassage.client.username') ?? '');
const currentPassword = ref('');
const newPassword = ref('');
const confirmPassword = ref('');
const reconnectDrives = ref(true);
const loading = ref(false);
const message = ref('');
const error = ref('');

const drives = reactive<DriveMapping[]>([
  { letter: 'S:', remote_path: localStorage.getItem('winpassage.client.drive.S') ?? '\\\\CENTRAL-PC\\Shared' },
  { letter: 'I:', remote_path: localStorage.getItem('winpassage.client.drive.I') ?? '\\\\CENTRAL-PC\\Internal' },
  { letter: 'G:', remote_path: localStorage.getItem('winpassage.client.drive.G') ?? '\\\\CENTRAL-PC\\Groups' },
]);

const canSubmit = computed(() => {
  return username.value.trim().length > 0
    && currentPassword.value.length > 0
    && newPassword.value.length >= 12
    && newPassword.value === confirmPassword.value;
});

function persistSettings() {
  localStorage.setItem('winpassage.client.serverUrl', serverUrl.value.replace(/\/$/, ''));
  localStorage.setItem('winpassage.client.username', username.value.trim());
  for (const drive of drives) {
    localStorage.setItem(`winpassage.client.drive.${drive.letter.replace(':', '')}`, drive.remote_path);
  }
}

async function changePassword() {
  loading.value = true;
  message.value = '';
  error.value = '';
  persistSettings();

  try {
    const response = await fetch(`${serverUrl.value.replace(/\/$/, '')}/v1/me/password/change`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        username: username.value.trim(),
        current_password: currentPassword.value,
        new_password: newPassword.value,
        request_id: crypto.randomUUID(),
      }),
    });

    const payload = await response.json().catch(() => ({})) as PasswordChangeResponse & { error?: string };

    if (!response.ok) {
      throw new Error(payload.error ?? `Request failed with ${response.status}`);
    }

    let driveMessage = '';
    if (reconnectDrives.value) {
      const selectedDrives = drives.filter((drive) => drive.letter.trim() && drive.remote_path.trim());
      const reconnect = await invoke<ReconnectMappedDrivesResponse>('reconnect_mapped_drives', {
        request: {
          username: username.value.trim(),
          password: newPassword.value,
          drives: selectedDrives,
        },
      });

      const ok = reconnect.results.filter((item) => item.success).length;
      driveMessage = ` Reconnected ${ok}/${reconnect.results.length} mapped drives.`;
    }

    currentPassword.value = '';
    newPassword.value = '';
    confirmPassword.value = '';
    message.value = `${payload.message}.${driveMessage}`;
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
        <h1>WinPassage Client</h1>
        <p>Change your central Windows password and refresh mapped drive credentials in one controlled flow.</p>
      </header>

      <p class="notice warning">Use this only on your company network or VPN. Never submit your password through a public or untrusted network.</p>

      <section class="card">
        <h2>Password change</h2>
        <div class="grid two">
          <label>
            Server URL
            <input v-model="serverUrl" placeholder="http://CENTRAL-PC:4487" />
          </label>
          <label>
            Username
            <input v-model="username" autocomplete="username" />
          </label>
          <label>
            Current password
            <input v-model="currentPassword" type="password" autocomplete="current-password" />
          </label>
          <label>
            New password
            <input v-model="newPassword" type="password" autocomplete="new-password" />
          </label>
          <label>
            Confirm new password
            <input v-model="confirmPassword" type="password" autocomplete="new-password" />
          </label>
        </div>
      </section>

      <section class="card">
        <h2>Mapped drives</h2>
        <label>
          <span>
            <input v-model="reconnectDrives" type="checkbox" style="width: auto; margin-right: .5rem;" />
            Reconnect mapped drives after successful password change
          </span>
        </label>

        <div class="grid" v-if="reconnectDrives">
          <div class="grid two" v-for="drive in drives" :key="drive.letter">
            <label>
              Drive letter
              <input v-model="drive.letter" />
            </label>
            <label>
              Remote path
              <input v-model="drive.remote_path" placeholder="\\CENTRAL-PC\Shared" />
            </label>
          </div>
        </div>
      </section>

      <p v-if="message" class="notice success">{{ message }}</p>
      <p v-if="error" class="notice error">{{ error }}</p>

      <section class="card">
        <div class="actions">
          <button :disabled="loading || !canSubmit" @click="changePassword">Change password</button>
          <button class="secondary" :disabled="loading" @click="persistSettings">Save settings</button>
        </div>
        <p class="notice">WinPassage never stores your password. Drive reconnect uses the new password only after the server confirms the password change.</p>
      </section>
    </section>
  </main>
</template>
