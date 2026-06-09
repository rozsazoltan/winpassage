<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, nextTick, reactive, ref } from 'vue';
import { driver } from 'driver.js';
import 'driver.js/dist/driver.css';
import { loadStoredDrives, normalizeDriveLetter } from './lib/driveSettings';
import type {
  DriveMapping,
  DriveOperationResponse,
  PasswordChangeResponse,
  ReconnectMappedDrivesResponse,
} from './types';

interface DriveConfig extends DriveMapping {
  id: string;
  status?: string;
  busy?: boolean;
}

const SERVER_STORAGE_KEY = 'winpassage.client.serverUrl';
const DRIVES_STORAGE_KEY = 'winpassage.client.drives';
const DEFAULT_SERVER_URL = 'http://CENTRAL-PC:4487';

function normalizeServerUrl(value: string): string {
  const trimmed = value.trim();
  return (trimmed.includes('://') ? trimmed : `http://${trimmed}`).replace(/\/+$/, '');
}

function loadDrives(): DriveConfig[] {
  return loadStoredDrives(localStorage.getItem(DRIVES_STORAGE_KEY)).map((drive) => ({
    ...drive,
    busy: false,
  }));
}

const serverUrl = ref(localStorage.getItem(SERVER_STORAGE_KEY) ?? DEFAULT_SERVER_URL);
const username = ref(localStorage.getItem('winpassage.client.username') ?? '');
const currentPassword = ref('');
const newPassword = ref('');
const confirmPassword = ref('');
const reconnectDrives = ref(true);
const drivePassword = ref('');
const loading = ref(false);
const message = ref('');
const error = ref('');
const connectionSettingsOpen = ref(false);
const connectionUnlockInput = ref('');
const pendingServerUrl = ref(serverUrl.value);
const setupOpen = ref(localStorage.getItem('winpassage.client.initialSetupComplete') !== 'true');
const tourCompleted = ref(localStorage.getItem('winpassage.client.tourComplete') === 'true');

const drives = reactive<DriveConfig[]>(loadDrives());
const newDriveLetter = ref('');
const newDrivePath = ref('');

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
  if (!newPassword.value) return 'Enter a new password.';
  if (newPassword.value.length < 12) return 'Use at least 12 characters.';
  if (newPassword.value !== confirmPassword.value) return 'Confirmation does not match.';
  if (passwordScore.value >= 80) return 'Strong password.';
  return 'Add more variety if possible.';
});

const configuredDriveCount = computed(() => drives.length);
const canEditConnection = computed(() => connectionUnlockInput.value.trim() === 'CHANGE SERVER');
const connectionLabel = computed(() => serverUrl.value.replace(/^https?:\/\//, ''));
const driveCredential = computed(() => drivePassword.value || newPassword.value || currentPassword.value);

const canSubmit = computed(() => {
  return username.value.trim().length > 0
    && currentPassword.value.length > 0
    && newPassword.value.length >= 12
    && newPassword.value === confirmPassword.value
    && serverUrl.value.trim().length > 0;
});

function persistSettings() {
  localStorage.setItem(SERVER_STORAGE_KEY, normalizeServerUrl(serverUrl.value));
  localStorage.setItem('winpassage.client.username', username.value.trim());
  persistDrives();
}

function persistDrives() {
  localStorage.setItem(
    DRIVES_STORAGE_KEY,
    JSON.stringify(drives.map(({ id, letter, remote_path }) => ({ id, letter, remote_path }))),
  );
}

function saveInitialSetup() {
  serverUrl.value = normalizeServerUrl(serverUrl.value);
  persistSettings();
  localStorage.setItem('winpassage.client.initialSetupComplete', 'true');
  setupOpen.value = false;
  void nextTick(() => startClientTour());
}

function startClientTour(force = false) {
  if (!force && tourCompleted.value) return;

  const tour = driver({
    showProgress: true,
    allowClose: true,
    nextBtnText: 'Next',
    prevBtnText: 'Back',
    doneBtnText: 'Done',
    steps: [
      {
        element: '#client-connection',
        popover: {
          title: 'Server address',
          description: 'Clients connect with an IP:port address. Changes are behind advanced settings to avoid mistakes.',
        },
      },
      {
        element: '#client-password',
        popover: {
          title: 'Change password',
          description: 'The server verifies your current password before applying the new one.',
        },
      },
      {
        element: '#client-drives',
        popover: {
          title: 'Drive mappings',
          description: 'No drives are configured by default. Add only the mappings used on this workstation.',
        },
      },
    ],
    onDestroyed: () => {
      tourCompleted.value = true;
      localStorage.setItem('winpassage.client.tourComplete', 'true');
    },
  });

  tour.drive();
}

function openConnectionSettings() {
  connectionSettingsOpen.value = true;
  pendingServerUrl.value = serverUrl.value;
  connectionUnlockInput.value = '';
}

function saveConnectionSettings() {
  if (!canEditConnection.value) return;

  try {
    serverUrl.value = normalizeServerUrl(pendingServerUrl.value);
    new URL(serverUrl.value);
    persistSettings();
    connectionSettingsOpen.value = false;
    connectionUnlockInput.value = '';
    message.value = 'Server address updated.';
    error.value = '';
  } catch {
    error.value = 'Invalid server address. Use an address such as http://192.168.1.10:4487.';
  }
}

function addDrive() {
  const letter = normalizeDriveLetter(newDriveLetter.value);
  const remotePath = newDrivePath.value.trim();
  if (!letter || !remotePath) {
    error.value = 'Drive letter and remote path are required.';
    return;
  }
  if (drives.some((drive) => drive.letter.toUpperCase() === letter)) {
    error.value = `${letter} is already configured.`;
    return;
  }

  drives.push({ id: crypto.randomUUID(), letter, remote_path: remotePath });
  newDriveLetter.value = '';
  newDrivePath.value = '';
  error.value = '';
  persistDrives();
}

function removeDrive(drive: DriveConfig) {
  const index = drives.findIndex((item) => item.id === drive.id);
  if (index >= 0) {
    drives.splice(index, 1);
    persistDrives();
  }
}

async function mountDrive(drive: DriveConfig) {
  if (!username.value.trim() || !driveCredential.value) {
    error.value = 'Username and a password are required to mount a drive.';
    return;
  }

  drive.busy = true;
  try {
    const result = await invoke<DriveOperationResponse>('mount_mapped_drive', {
      request: {
        username: username.value.trim(),
        password: driveCredential.value,
        drive: { letter: drive.letter, remote_path: drive.remote_path },
      },
    });
    drive.status = result.message;
    if (!result.success) throw new Error(result.message);
    message.value = `${drive.letter} mounted.`;
    error.value = '';
  } catch (caught) {
    drive.status = caught instanceof Error ? caught.message : String(caught);
    error.value = drive.status;
  } finally {
    drive.busy = false;
  }
}

async function unmountDrive(drive: DriveConfig) {
  drive.busy = true;
  try {
    const result = await invoke<DriveOperationResponse>('unmount_mapped_drive', { letter: drive.letter });
    drive.status = result.message;
    if (!result.success) throw new Error(result.message);
    message.value = `${drive.letter} unmounted.`;
    error.value = '';
  } catch (caught) {
    drive.status = caught instanceof Error ? caught.message : String(caught);
    error.value = drive.status;
  } finally {
    drive.busy = false;
  }
}

async function changePassword() {
  loading.value = true;
  message.value = '';
  error.value = '';
  persistSettings();

  try {
    const response = await fetch(`${normalizeServerUrl(serverUrl.value)}/v1/me/password/change`, {
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
    if (reconnectDrives.value && drives.length > 0) {
      const reconnect = await invoke<ReconnectMappedDrivesResponse>('reconnect_mapped_drives', {
        request: {
          username: username.value.trim(),
          password: newPassword.value,
          drives: drives.map(({ letter, remote_path }) => ({ letter, remote_path })),
        },
      });

      const ok = reconnect.results.filter((item) => item.success).length;
      driveMessage = ` Reconnected ${ok}/${reconnect.results.length} drives.`;
    }

    drivePassword.value = newPassword.value;
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

if (!setupOpen.value) {
  void nextTick(() => startClientTour());
}
</script>

<template>
  <main class="app app-client">
    <section v-if="setupOpen" class="setup-overlay" role="dialog" aria-modal="true">
      <article class="setup-card">
        <p class="eyebrow">First run</p>
        <h1>Connect this client.</h1>
        <p class="lead">Enter the server IP:port and your Windows username before the guide starts.</p>
        <div class="grid two">
          <label>
            Server URL
            <input v-model="serverUrl" placeholder="http://192.168.1.10:4487" />
          </label>
          <label>
            Username
            <input v-model="username" autocomplete="username" placeholder="julia" />
          </label>
        </div>
        <div class="actions">
          <button :disabled="!serverUrl.trim() || !username.trim()" @click="saveInitialSetup">Save and show guide</button>
        </div>
      </article>
    </section>

    <section class="app-shell">
      <aside class="side-rail" aria-label="WinPassageClient navigation">
        <div class="brand-block">
          <div class="brand-mark" aria-hidden="true"><span class="bridge-icon">⊞</span></div>
          <div>
            <p class="brand-title">WinPassageClient</p>
            <p class="brand-subtitle">Password self-service</p>
          </div>
        </div>

        <nav class="rail-menu" aria-label="Workflow sections">
          <span class="rail-item active"><span class="rail-dot"></span>Password</span>
          <span class="rail-item">Drives</span>
          <span class="rail-item">Settings</span>
        </nav>

        <div class="rail-footer">
          <p class="metric-label">Privacy</p>
          <p class="muted">Passwords are used only for the current request.</p>
        </div>
      </aside>

      <section class="workspace">
        <header class="hero-panel">
          <div class="hero-grid">
            <div>
              <p class="eyebrow">Self-service</p>
              <h1>Change your Windows password.</h1>
              <p class="lead">Use your current password, choose a new one, then reconnect only the drives configured on this computer.</p>
            </div>
            <div class="status-stack">
              <button class="secondary" @click="startClientTour(true)">Show guide</button>
            </div>
          </div>
        </header>

        <p class="notice warning">Use this only on the company network or VPN.</p>
        <p v-if="message" class="notice success">{{ message }}</p>
        <p v-if="error" class="notice error">{{ error }}</p>

        <section class="metric-grid" aria-label="Status">
          <article class="metric-card">
            <p class="metric-label">Server</p>
            <p class="metric-value compact">{{ connectionLabel }}</p>
            <p class="metric-hint">IP:port</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Account</p>
            <p class="metric-value">{{ username || '—' }}</p>
            <p class="metric-hint">Windows username</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Password</p>
            <p class="metric-value">{{ passwordScore }}%</p>
            <p class="metric-hint">{{ passwordStatus }}</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Drives</p>
            <p class="metric-value">{{ configuredDriveCount }}</p>
            <p class="metric-hint">No defaults</p>
          </article>
        </section>

        <section id="client-connection" class="surface-card connection-card">
          <div class="card-heading">
            <div>
              <h2>Connection</h2>
              <p class="muted">Server address changes are intentionally tucked away.</p>
            </div>
            <span class="status-pill">{{ connectionLabel }}</span>
          </div>

          <div v-if="!connectionSettingsOpen" class="connection-summary">
            <div>
              <p class="metric-label">Current server</p>
              <strong>{{ serverUrl }}</strong>
            </div>
            <button class="secondary" @click="openConnectionSettings">Advanced settings</button>
          </div>

          <div v-else class="settings-gate">
            <p class="notice warning">Only change this when IT gives you a new IP:port.</p>
            <label>
              Type CHANGE SERVER
              <input v-model="connectionUnlockInput" autocomplete="off" placeholder="CHANGE SERVER" />
            </label>
            <label>
              Server URL
              <input v-model="pendingServerUrl" :disabled="!canEditConnection" placeholder="http://192.168.1.10:4487" />
            </label>
            <div class="actions">
              <button :disabled="!canEditConnection" @click="saveConnectionSettings">Save</button>
              <button class="secondary" @click="connectionSettingsOpen = false">Cancel</button>
            </div>
          </div>
        </section>

        <section class="panel-grid reverse">
          <article id="client-password" class="surface-card">
            <div class="card-heading">
              <div>
                <h2>Password</h2>
                <p class="muted">The server verifies the current password first.</p>
              </div>
              <span class="status-pill" :class="canSubmit ? 'success' : 'warning'">{{ canSubmit ? 'Ready' : 'Incomplete' }}</span>
            </div>

            <div class="grid two">
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

            <div class="password-meter" aria-label="Password strength">
              <span :style="{ width: `${passwordScore}%` }"></span>
            </div>
            <p class="muted">{{ passwordStatus }}</p>

            <label class="toggle-row">
              <input v-model="reconnectDrives" type="checkbox" />
              Reconnect configured drives after password change
            </label>

            <div class="actions">
              <button :disabled="loading || !canSubmit" @click="changePassword">Change password</button>
            </div>
          </article>

          <article id="client-drives" class="surface-card">
            <div class="card-heading">
              <div>
                <h2>Drive mappings</h2>
                <p class="muted">Add only the drives this workstation needs.</p>
              </div>
              <span class="status-pill">{{ configuredDriveCount }}</span>
            </div>

            <div class="grid three">
              <label>
                Letter
                <input v-model="newDriveLetter" placeholder="S:" />
              </label>
              <label>
                Remote path
                <input v-model="newDrivePath" placeholder="\\\\SERVER\\Share" />
              </label>
              <div class="field-actions">
                <button class="secondary" @click="addDrive">Add drive</button>
              </div>
            </div>

            <label>
              Drive password for manual mount
              <input v-model="drivePassword" type="password" autocomplete="current-password" placeholder="Optional; otherwise current/new password is used" />
            </label>

            <div v-if="drives.length === 0" class="empty-state">
              <strong>No drive mappings configured.</strong>
              <p class="muted">WinPassage starts with zero default drives.</p>
            </div>

            <div v-else class="drive-list">
              <article v-for="drive in drives" :key="drive.id" class="drive-card">
                <div class="grid two compact-grid">
                  <label>
                    Letter
                    <input v-model="drive.letter" @change="persistDrives" />
                  </label>
                  <label>
                    Remote path
                    <input v-model="drive.remote_path" @change="persistDrives" />
                  </label>
                </div>
                <p v-if="drive.status" class="muted">{{ drive.status }}</p>
                <div class="actions wrap">
                  <button class="secondary small" :disabled="drive.busy" @click="mountDrive(drive)">Mount</button>
                  <button class="secondary small" :disabled="drive.busy" @click="unmountDrive(drive)">Unmount</button>
                  <button class="danger small" :disabled="drive.busy" @click="removeDrive(drive)">Delete</button>
                </div>
              </article>
            </div>
          </article>
        </section>
      </section>
    </section>
  </main>
</template>
