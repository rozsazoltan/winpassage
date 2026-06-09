<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, nextTick, onMounted, reactive, ref } from 'vue';
import { driver } from 'driver.js';
import 'driver.js/dist/driver.css';
import { winPassageMarkSvg } from './brand';
import { lucideSvg, type IconName } from './icons';
import { loadStoredDrives, normalizeDriveLetter } from './lib/driveSettings';
import type { DriveMapping, DriveOperationResponse, PasswordChangeResponse, ReconnectMappedDrivesResponse } from './types';

type ClientTab = 'overview' | 'password' | 'drives' | 'settings';
type ThemeMode = 'light' | 'system' | 'dark';

interface DriveConfig extends DriveMapping {
  id: string;
  name?: string;
  status?: string;
  mounted?: boolean;
  busy?: boolean;
}

const SERVER_STORAGE_KEY = 'winpassage.client.serverUrl';
const USERNAME_STORAGE_KEY = 'winpassage.client.username';
const DRIVES_STORAGE_KEY = 'winpassage.client.drives';
const THEME_KEY = 'winpassage.client.theme';
const SETUP_KEY = 'winpassage.client.initialSetupComplete';
const TOUR_KEY = 'winpassage.client.tourComplete';
const DEFAULT_SERVER_URL = 'http://CENTRAL-PC:4487';

const navigation: Array<{ id: ClientTab; label: string; icon: IconName }> = [
  { id: 'overview', label: 'Overview', icon: 'overview' },
  { id: 'password', label: 'Password', icon: 'password' },
  { id: 'drives', label: 'Drives', icon: 'drives' },
  { id: 'settings', label: 'Settings', icon: 'settings' },
];

function icon(name: IconName): string {
  return lucideSvg(name);
}

function normalizeServerUrl(value: string): string {
  const trimmed = value.trim();
  return (trimmed.includes('://') ? trimmed : `http://${trimmed}`).replace(/\/+$/, '');
}

function loadDrives(): DriveConfig[] {
  return loadStoredDrives(localStorage.getItem(DRIVES_STORAGE_KEY)).map((drive) => ({ ...drive, busy: false, mounted: false }));
}

const brandIcon = winPassageMarkSvg('brand-svg');
const activeTab = ref<ClientTab>('overview');
const theme = ref<ThemeMode>((localStorage.getItem(THEME_KEY) as ThemeMode | null) ?? 'system');
const serverUrl = ref(localStorage.getItem(SERVER_STORAGE_KEY) ?? DEFAULT_SERVER_URL);
const username = ref(localStorage.getItem(USERNAME_STORAGE_KEY) ?? '');
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
const setupOpen = ref(localStorage.getItem(SETUP_KEY) !== 'true');
const tourCompleted = ref(localStorage.getItem(TOUR_KEY) === 'true');
const driveEditorOpen = ref(false);
const editingDriveId = ref<string | null>(null);

const drives = reactive<DriveConfig[]>(loadDrives());
const driveFormName = ref('');
const driveFormLetter = ref('');
const driveFormPath = ref('');

const currentPage = computed(() => navigation.find((item) => item.id === activeTab.value) ?? navigation[0]);
const configuredDriveCount = computed(() => drives.length);
const connectionLabel = computed(() => serverUrl.value.replace(/^https?:\/\//, ''));
const canEditConnection = computed(() => connectionUnlockInput.value.trim() === 'CHANGE SERVER');
const driveCredential = computed(() => drivePassword.value || newPassword.value || currentPassword.value);

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

const canSubmit = computed(
  () => username.value.trim().length > 0 && currentPassword.value.length > 0 && newPassword.value.length >= 12 && newPassword.value === confirmPassword.value && serverUrl.value.trim().length > 0,
);

const canSaveDrive = computed(() => normalizeDriveLetter(driveFormLetter.value) !== null && driveFormPath.value.trim().length > 0);

function applyTheme() {
  const resolved = theme.value === 'system' ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light') : theme.value;
  document.documentElement.dataset.theme = resolved;
  localStorage.setItem(THEME_KEY, theme.value);
}

function setTheme(value: ThemeMode) {
  theme.value = value;
  applyTheme();
}

function persistDrives() {
  localStorage.setItem(DRIVES_STORAGE_KEY, JSON.stringify(drives.map(({ id, letter, remote_path }) => ({ id, letter, remote_path }))));
}

function persistSettings() {
  serverUrl.value = normalizeServerUrl(serverUrl.value);
  localStorage.setItem(SERVER_STORAGE_KEY, serverUrl.value);
  localStorage.setItem(USERNAME_STORAGE_KEY, username.value.trim());
  persistDrives();
}

function saveInitialSetup() {
  persistSettings();
  localStorage.setItem(SETUP_KEY, 'true');
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
      { element: '#client-connection', popover: { title: 'Server connection', description: 'Clients connect using the configured IP:port endpoint.' } },
      { element: '#client-password', popover: { title: 'Password change', description: 'Your current password is verified by the central machine first.' } },
      { element: '#client-drives', popover: { title: 'Drive mappings', description: 'No default drives are configured. Add only what this workstation needs.' } },
      { element: '#client-settings', popover: { title: 'Settings', description: 'Server changes stay behind advanced confirmation to avoid user mistakes.' } },
    ],
    onDestroyed: () => {
      tourCompleted.value = true;
      localStorage.setItem(TOUR_KEY, 'true');
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
    error.value = 'Use a server address such as http://192.168.1.10:4487.';
  }
}

function openDriveEditor(drive?: DriveConfig) {
  editingDriveId.value = drive?.id ?? null;
  driveFormName.value = drive?.name ?? '';
  driveFormLetter.value = drive?.letter ?? '';
  driveFormPath.value = drive?.remote_path ?? '';
  driveEditorOpen.value = true;
}

function closeDriveEditor() {
  editingDriveId.value = null;
  driveFormName.value = '';
  driveFormLetter.value = '';
  driveFormPath.value = '';
  driveEditorOpen.value = false;
}

function saveDrive() {
  const letter = normalizeDriveLetter(driveFormLetter.value);
  const remotePath = driveFormPath.value.trim();
  if (!letter || !remotePath) {
    error.value = 'Drive letter and remote path are required.';
    return;
  }

  const duplicate = drives.some((drive) => drive.id !== editingDriveId.value && drive.letter.toUpperCase() === letter);
  if (duplicate) {
    error.value = `${letter} is already configured.`;
    return;
  }

  if (editingDriveId.value) {
    const drive = drives.find((item) => item.id === editingDriveId.value);
    if (drive) {
      drive.name = driveFormName.value.trim() || undefined;
      drive.letter = letter;
      drive.remote_path = remotePath;
    }
  } else {
    drives.push({ id: crypto.randomUUID(), name: driveFormName.value.trim() || undefined, letter, remote_path: remotePath, busy: false, mounted: false });
  }

  persistDrives();
  closeDriveEditor();
  error.value = '';
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
      request: { username: username.value.trim(), password: driveCredential.value, drive: { letter: drive.letter, remote_path: drive.remote_path } },
    });
    drive.status = result.message;
    drive.mounted = result.success;
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
    drive.mounted = false;
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
      body: JSON.stringify({ username: username.value.trim(), current_password: currentPassword.value, new_password: newPassword.value, request_id: crypto.randomUUID() }),
    });
    const payload = (await response.json().catch(() => ({}))) as PasswordChangeResponse & { error?: string };
    if (!response.ok) throw new Error(payload.error ?? `Password change failed with ${response.status}`);

    let driveMessage = '';
    if (reconnectDrives.value && drives.length > 0) {
      const reconnect = await invoke<ReconnectMappedDrivesResponse>('reconnect_mapped_drives', {
        request: { username: username.value.trim(), password: newPassword.value, drives: drives.map(({ letter, remote_path }) => ({ letter, remote_path })) },
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

onMounted(() => {
  applyTheme();
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyTheme);
  if (!setupOpen.value) void nextTick(() => startClientTour());
});
</script>

<template>
  <main class="app app-client">
    <section v-if="setupOpen" class="setup-overlay" role="dialog" aria-modal="true">
      <article class="setup-card">
        <div class="setup-brand" v-html="brandIcon"></div>
        <p class="eyebrow">Initial setup</p>
        <h1>Prepare WinPassageClient</h1>
        <p class="lead">Set the server endpoint and username. The guided tour starts after setup.</p>
        <div class="form-grid two">
          <label>Server endpoint<input v-model="serverUrl" placeholder="http://192.168.1.10:4487" /></label>
          <label>Windows username<input v-model="username" autocomplete="username" placeholder="username" /></label>
        </div>
        <div class="button-row end">
          <button class="btn-primary" :disabled="!serverUrl.trim() || !username.trim()" @click="saveInitialSetup">Save and continue</button>
        </div>
      </article>
    </section>

    <section class="app-shell">
      <aside class="sidebar" aria-label="WinPassageClient navigation">
        <div class="brand-row">
          <span class="app-mark" v-html="brandIcon"></span>
          <div>
            <p class="brand-title">WinPassageClient</p>
            <p class="brand-subtitle">Secure access</p>
          </div>
        </div>

        <nav class="nav-list" aria-label="Main sections">
          <button v-for="item in navigation" :key="item.id" class="nav-item" :class="{ active: activeTab === item.id }" @click="activeTab = item.id">
            <span v-html="icon(item.icon)"></span>{{ item.label }}
          </button>
        </nav>

        <div class="sidebar-status" id="client-connection">
          <p class="status-line"><span class="dot success"></span>Connected</p>
          <p>{{ connectionLabel }}</p>
        </div>
      </aside>

      <section class="workspace">
        <header class="topbar">
          <div>
            <p class="eyebrow">{{ currentPage.label }}</p>
            <h1>{{ currentPage.label }}</h1>
          </div>
          <div class="topbar-actions">
            <button class="btn-ghost" @click="setupOpen = true"><span v-html="icon('settings')"></span>Setup</button>
            <button class="btn-ghost" @click="startClientTour(true)"><span v-html="icon('play')"></span>Guide</button>
          </div>
        </header>

        <p v-if="message" class="notice success"><span v-html="icon('check')"></span>{{ message }}</p>
        <p v-if="error" class="notice error"><span v-html="icon('warning')"></span>{{ error }}</p>

        <section v-if="activeTab === 'overview'" class="page-stack">
          <article class="surface-card" id="client-password">
            <div class="card-title-row">
              <div>
                <h2>Self-service access</h2>
                <p>Change your Windows password and keep mapped drives in sync.</p>
              </div>
              <span class="hero-mark" v-html="brandIcon"></span>
            </div>
            <ol class="setup-list">
              <li class="done"><span v-html="icon('check')"></span>Server <strong>{{ connectionLabel }}</strong></li>
              <li :class="{ done: username }"><span v-html="icon(username ? 'check' : 'info')"></span>User <strong>{{ username || 'Not set' }}</strong></li>
              <li><span v-html="icon('info')"></span>Drive mappings <strong>{{ configuredDriveCount }} configured</strong></li>
            </ol>
            <div class="button-row">
              <button class="btn-primary" @click="activeTab = 'password'">Change password</button>
              <button class="btn-secondary" @click="startClientTour(true)"><span v-html="icon('play')"></span>Start tour</button>
            </div>
          </article>

          <div class="page-grid two">
            <article class="surface-card">
              <h2>Connection</h2>
              <div class="stat-list">
                <span>Endpoint</span><strong>{{ serverUrl }}</strong>
                <span>Username</span><strong>{{ username || '—' }}</strong>
              </div>
            </article>
            <article class="surface-card">
              <h2>Drives</h2>
              <div class="stat-list">
                <span>Configured</span><strong>{{ configuredDriveCount }}</strong>
                <span>Default drives</span><strong>0</strong>
              </div>
              <button class="btn-secondary" @click="activeTab = 'drives'">Manage drives</button>
            </article>
          </div>
        </section>

        <section v-if="activeTab === 'password'" class="page-grid two">
          <article class="surface-card">
            <h2>Change password</h2>
            <p class="muted">The central machine verifies your current password first.</p>
            <div class="form-grid two">
              <label>Username<input v-model="username" autocomplete="username" /></label>
              <label>Current password<input v-model="currentPassword" type="password" autocomplete="current-password" /></label>
              <label>New password<input v-model="newPassword" type="password" autocomplete="new-password" /></label>
              <label>Confirm password<input v-model="confirmPassword" type="password" autocomplete="new-password" /></label>
            </div>
            <div class="password-meter"><span :style="{ width: `${passwordScore}%` }"></span></div>
            <p class="muted">{{ passwordStatus }}</p>
            <label class="check-row"><input v-model="reconnectDrives" type="checkbox" />Reconnect configured drives</label>
            <button class="btn-primary" :disabled="loading || !canSubmit" @click="changePassword">Change password</button>
          </article>

          <article class="surface-card">
            <h2>After success</h2>
            <p class="muted">WinPassage can update only the drive mappings you configured.</p>
            <div class="stat-list"><span>Server</span><strong>{{ connectionLabel }}</strong><span>Drives</span><strong>{{ configuredDriveCount }}</strong></div>
          </article>
        </section>

        <section v-if="activeTab === 'drives'" id="client-drives" class="page-stack">
          <article class="surface-card table-card">
            <div class="card-title-row">
              <div><h2>Drive mappings</h2><p>No mappings are created by default.</p></div>
              <button class="btn-primary" @click="openDriveEditor()"><span v-html="icon('plus')"></span>Add drive</button>
            </div>
            <div v-if="drives.length === 0" class="empty-state">
              <span v-html="brandIcon"></span>
              <h3>No drives configured</h3>
              <p>Add only the network drives this workstation needs.</p>
            </div>
            <div v-else class="table-wrap">
              <table class="data-table">
                <thead><tr><th>Name</th><th>Letter</th><th>Remote path</th><th>Status</th><th>Actions</th></tr></thead>
                <tbody>
                  <tr v-for="drive in drives" :key="drive.id">
                    <td><strong>{{ drive.name || 'Network drive' }}</strong></td>
                    <td>{{ drive.letter }}</td>
                    <td>{{ drive.remote_path }}</td>
                    <td><span class="pill" :class="drive.mounted ? 'success' : 'neutral'">{{ drive.mounted ? 'Mounted' : 'Not mounted' }}</span></td>
                    <td class="row-actions">
                      <button class="icon-btn" :disabled="drive.busy" title="Mount" @click="mountDrive(drive)"><span v-html="icon('play')"></span></button>
                      <button class="icon-btn" :disabled="drive.busy" title="Unmount" @click="unmountDrive(drive)"><span v-html="icon('stop')"></span></button>
                      <button class="icon-btn" :disabled="drive.busy" title="Edit" @click="openDriveEditor(drive)"><span v-html="icon('edit')"></span></button>
                      <button class="icon-btn danger" :disabled="drive.busy" title="Delete" @click="removeDrive(drive)"><span v-html="icon('trash')"></span></button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </article>

          <details class="advanced">
            <summary>Mount credentials</summary>
            <div class="form-grid">
              <p class="muted">Optional. If empty, the current session password is used when available.</p>
              <label>Drive password<input v-model="drivePassword" type="password" autocomplete="current-password" /></label>
            </div>
          </details>
        </section>

        <section v-if="activeTab === 'settings'" id="client-settings" class="page-grid two">
          <article class="surface-card">
            <h2>Appearance</h2>
            <div class="segmented">
              <button :class="{ active: theme === 'light' }" @click="setTheme('light')"><span v-html="icon('sun')"></span>Light</button>
              <button :class="{ active: theme === 'system' }" @click="setTheme('system')"><span v-html="icon('monitor')"></span>System</button>
              <button :class="{ active: theme === 'dark' }" @click="setTheme('dark')"><span v-html="icon('moon')"></span>Dark</button>
            </div>
          </article>

          <article class="surface-card">
            <h2>Server connection</h2>
            <p class="muted">Change only when an administrator gives you a new IP:port.</p>
            <div v-if="!connectionSettingsOpen" class="button-row between"><strong>{{ serverUrl }}</strong><button class="btn-secondary" @click="openConnectionSettings">Advanced edit</button></div>
            <div v-else class="form-grid">
              <label>Type CHANGE SERVER<input v-model="connectionUnlockInput" placeholder="CHANGE SERVER" /></label>
              <label>Server URL<input v-model="pendingServerUrl" :disabled="!canEditConnection" /></label>
              <div class="button-row"><button class="btn-primary" :disabled="!canEditConnection" @click="saveConnectionSettings">Save</button><button class="btn-secondary" @click="connectionSettingsOpen = false">Cancel</button></div>
            </div>
          </article>

          <article class="surface-card">
            <h2>Guided tour</h2>
            <p class="muted">Replay onboarding at any time.</p>
            <button class="btn-secondary" @click="startClientTour(true)">Replay tour</button>
          </article>
        </section>
      </section>
    </section>

    <section v-if="driveEditorOpen" class="drawer-backdrop" @click.self="closeDriveEditor">
      <aside class="drawer" role="dialog" aria-modal="true">
        <div class="card-title-row">
          <div><h2>{{ editingDriveId ? 'Edit drive' : 'Add drive' }}</h2><p>Map one network location.</p></div>
          <button class="icon-btn" @click="closeDriveEditor"><span v-html="icon('x')"></span></button>
        </div>
        <div class="form-grid">
          <label>Name<input v-model="driveFormName" placeholder="Projects" /></label>
          <label>Drive letter<input v-model="driveFormLetter" placeholder="S:" /></label>
          <label>Remote path<input v-model="driveFormPath" placeholder="\\SERVER\Share" /></label>
        </div>
        <div class="button-row end"><button class="btn-secondary" @click="closeDriveEditor">Cancel</button><button class="btn-primary" :disabled="!canSaveDrive" @click="saveDrive">Save drive</button></div>
      </aside>
    </section>
  </main>
</template>
