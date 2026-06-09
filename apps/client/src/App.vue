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

const passwordScore = computed(() => {
  let score = 0;
  if (newPassword.value.length >= 12) score += 35;
  if (/[A-Z]/.test(newPassword.value)) score += 15;
  if (/[a-z]/.test(newPassword.value)) score += 15;
  if (/\d/.test(newPassword.value)) score += 15;
  if (/[^A-Za-z0-9]/.test(newPassword.value)) score += 20;
  return Math.min(score, 100);
});

const passwordStatus = computed(() => {
  if (!newPassword.value) return 'Waiting for new password';
  if (newPassword.value.length < 12) return 'Minimum 12 characters required';
  if (newPassword.value !== confirmPassword.value) return 'Confirmation does not match';
  if (passwordScore.value >= 80) return 'Strong password shape';
  return 'Acceptable, but add more variety if possible';
});

const configuredDriveCount = computed(() => drives.filter((drive) => drive.letter.trim() && drive.remote_path.trim()).length);

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
  <main class="app app-client">
    <section class="app-shell">
      <aside class="side-rail" aria-label="WinPassage Client navigation">
        <div class="brand-block">
          <div class="brand-mark">WP</div>
          <div>
            <p class="brand-title">WinPassage</p>
            <p class="brand-subtitle">Self-service</p>
          </div>
        </div>

        <nav class="rail-menu" aria-label="Workflow sections">
          <span class="rail-item active"><span class="rail-dot"></span>Password</span>
          <span class="rail-item">Drives</span>
          <span class="rail-item">Confirm</span>
          <span class="rail-item">Done</span>
        </nav>

        <div class="rail-footer">
          <p class="metric-label">Privacy</p>
          <p class="muted">Passwords are used for this request only and are not saved by the client app.</p>
        </div>
      </aside>

      <section class="workspace">
        <header class="hero-panel">
          <div class="hero-grid">
            <div>
              <p class="eyebrow">User password self-service</p>
              <h1>Change your password and keep your drives connected.</h1>
              <p class="lead">
                Update your central Windows password through the approved flow. After confirmation, WinPassage can refresh your mapped drive credentials automatically.
              </p>
            </div>
            <div class="status-stack">
              <span class="status-pill success">Own account only</span>
              <span class="status-pill warning">VPN or LAN</span>
              <span class="status-pill">No password storage</span>
            </div>
          </div>
        </header>

        <p class="notice warning">Use this only on the company network or VPN. Do not submit credentials through public or untrusted networks.</p>
        <p v-if="message" class="notice success">{{ message }}</p>
        <p v-if="error" class="notice error">{{ error }}</p>

        <section class="metric-grid" aria-label="Workflow status">
          <article class="metric-card">
            <p class="metric-label">Account</p>
            <p class="metric-value">{{ username || '—' }}</p>
            <p class="metric-hint">Central Windows username</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Password score</p>
            <p class="metric-value">{{ passwordScore }}%</p>
            <p class="metric-hint">Client-side guidance only</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Drive mappings</p>
            <p class="metric-value">{{ configuredDriveCount }}</p>
            <p class="metric-hint">Configured for reconnect</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Ready</p>
            <p class="metric-value">{{ canSubmit ? 'Yes' : 'No' }}</p>
            <p class="metric-hint">All required fields valid</p>
          </article>
        </section>

        <section class="panel-grid reverse">
          <article class="surface-card">
            <div class="card-heading">
              <div>
                <h2>Password change</h2>
                <p class="muted">The server verifies your current password before changing it.</p>
              </div>
              <span class="status-pill" :class="canSubmit ? 'success' : 'warning'">{{ canSubmit ? 'Ready' : 'Needs input' }}</span>
            </div>

            <div class="grid two">
              <label>
                Server URL
                <input v-model="serverUrl" placeholder="http://CENTRAL-PC:4487" />
              </label>
              <label>
                Username
                <input v-model="username" autocomplete="username" placeholder="your central username" />
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

            <div class="password-meter" aria-label="Password strength guidance">
              <div class="meter-track">
                <div class="meter-fill" :style="{ '--meter': `${passwordScore}%` }"></div>
              </div>
              <p class="micro-copy">{{ passwordStatus }}</p>
            </div>
          </article>

          <article class="surface-card">
            <div class="card-heading">
              <div>
                <h2>Drive reconnect</h2>
                <p class="muted">Refresh remembered SMB credentials after the server confirms success.</p>
              </div>
              <span class="status-pill">{{ configuredDriveCount }} drives</span>
            </div>

            <label class="check-row">
              <input v-model="reconnectDrives" type="checkbox" />
              Reconnect mapped drives after successful password change
            </label>

            <div v-if="reconnectDrives" class="drive-list">
              <div class="drive-card" v-for="drive in drives" :key="drive.letter">
                <label>
                  Drive
                  <input v-model="drive.letter" placeholder="S:" />
                </label>
                <label>
                  Remote path
                  <input v-model="drive.remote_path" placeholder="\\CENTRAL-PC\Shared" />
                </label>
              </div>
            </div>
          </article>
        </section>

        <section class="surface-card">
          <div class="card-heading">
            <div>
              <h2>Submit request</h2>
              <p class="muted">The client sends the password change first. Drive reconnect starts only after a successful server response.</p>
            </div>
            <span class="status-pill">Request ID generated automatically</span>
          </div>

          <div class="step-list">
            <div class="step-item">
              <span class="step-number">1</span>
              <div>
                <h3>Verify current password</h3>
                <p class="muted">The central Windows machine validates the existing username and password pair.</p>
              </div>
            </div>
            <div class="step-item">
              <span class="step-number">2</span>
              <div>
                <h3>Change password</h3>
                <p class="muted">The server changes only the authenticated user's own local account password.</p>
              </div>
            </div>
            <div class="step-item">
              <span class="step-number">3</span>
              <div>
                <h3>Reconnect drives</h3>
                <p class="muted">The client refreshes configured mapped drives with the new credentials when enabled.</p>
              </div>
            </div>
          </div>

          <div class="form-footer">
            <p class="micro-copy">Only server URL, username, and drive paths are stored locally.</p>
            <div class="actions">
              <button :disabled="loading || !canSubmit" @click="changePassword">Change password</button>
              <button class="secondary" :disabled="loading" @click="persistSettings">Save settings</button>
            </div>
          </div>
        </section>
      </section>
    </section>
  </main>
</template>
