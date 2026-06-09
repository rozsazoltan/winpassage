<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, nextTick, onMounted, ref } from 'vue';
import { driver } from 'driver.js';
import 'driver.js/dist/driver.css';
import { winPassageMarkSvg } from './brand';
import { lucideSvg, type IconName } from './icons';
import type {
  ActionResponse,
  AdminHostStatus,
  ListSessionsResponse,
  ListUsersResponse,
  LocalSessionSummary,
  LocalUserSummary,
  ServerInstallResult,
  ServerProfile,
} from './types';

type AdminTab = 'overview' | 'users' | 'sessions' | 'servers' | 'settings';
type ThemeMode = 'light' | 'system' | 'dark';

const PROFILE_STORAGE_KEY = 'winpassage.admin.serverProfiles';
const ACTIVE_PROFILE_KEY = 'winpassage.admin.activeServerId';
const THEME_KEY = 'winpassage.admin.theme';
const SETUP_KEY = 'winpassage.admin.initialSetupComplete';
const TOUR_KEY = 'winpassage.admin.tourComplete';

const navigation: Array<{ id: AdminTab; label: string; icon: IconName }> = [
  { id: 'overview', label: 'Overview', icon: 'overview' },
  { id: 'users', label: 'Users', icon: 'users' },
  { id: 'sessions', label: 'Sessions', icon: 'sessions' },
  { id: 'servers', label: 'Servers', icon: 'servers' },
  { id: 'settings', label: 'Settings', icon: 'settings' },
];

function defaultServerProfiles(): ServerProfile[] {
  return [
    {
      id: 'local-dev',
      name: 'Local machine',
      network_name: 'This device',
      protocol: 'http',
      host: 'localhost',
      port: 4487,
      notes: 'Default local WinPassage server profile.',
    },
  ];
}

function loadServerProfiles(): ServerProfile[] {
  const stored = localStorage.getItem(PROFILE_STORAGE_KEY);
  if (!stored) return defaultServerProfiles();

  try {
    const parsed = JSON.parse(stored) as ServerProfile[];
    return Array.isArray(parsed) && parsed.length > 0 ? parsed : defaultServerProfiles();
  } catch {
    return defaultServerProfiles();
  }
}

function profileUrl(profile: ServerProfile): string {
  return `${profile.protocol}://${profile.host}:${profile.port}`;
}

function normalizeUrl(value: string): URL {
  const trimmed = value.trim();
  return new URL(trimmed.includes('://') ? trimmed : `http://${trimmed}`);
}

function icon(name: IconName): string {
  return lucideSvg(name);
}

const brandIcon = winPassageMarkSvg('brand-svg');
const activeTab = ref<AdminTab>('overview');
const theme = ref<ThemeMode>((localStorage.getItem(THEME_KEY) as ThemeMode | null) ?? 'system');
const serverProfiles = ref<ServerProfile[]>(loadServerProfiles());
const activeServerId = ref(localStorage.getItem(ACTIVE_PROFILE_KEY) ?? serverProfiles.value[0]?.id ?? 'local-dev');
const serverUrl = ref(profileUrl(serverProfiles.value.find((profile) => profile.id === activeServerId.value) ?? serverProfiles.value[0]));
const adminToken = ref('');
const users = ref<LocalUserSummary[]>([]);
const sessions = ref<LocalSessionSummary[]>([]);
const selectedUser = ref('');
const newPassword = ref('');
const reason = ref('');
const loading = ref(false);
const message = ref('');
const error = ref('');

const hostStatus = ref<AdminHostStatus | null>(null);
const hostStatusLoading = ref(true);
const installSourceDir = ref('');
const installDir = ref('C:\\Program Files\\WinPassage');
const installBindHost = ref('0.0.0.0');
const installPort = ref(4487);
const installAdminToken = ref('');
const installRequireTls = ref(false);
const demoteConfirmation = ref('');
const demoteRemoveFiles = ref(true);

const newServerName = ref('');
const newServerNetwork = ref('');
const newServerHost = ref('');
const newServerPort = ref(4487);
const newServerProtocol = ref<'http' | 'https'>('http');
const newServerNotes = ref('');

const createUsername = ref('');
const createFullName = ref('');
const createPassword = ref('');
const createMustChange = ref(true);
const createEnabled = ref(true);
const createAdmin = ref(false);

const deleteConfirmation = ref('');
const deleteLogoffSessions = ref(true);
const deleteProfile = ref(false);

const initialSetupOpen = ref(localStorage.getItem(SETUP_KEY) !== 'true');
const tourCompleted = ref(localStorage.getItem(TOUR_KEY) === 'true');

const activeServerProfile = computed(
  () => serverProfiles.value.find((profile) => profile.id === activeServerId.value) ?? serverProfiles.value[0],
);
const enabledUsers = computed(() => users.value.filter((user) => !user.disabled).length);
const adminUsers = computed(() => users.value.filter((user) => user.is_administrator).length);
const activeSessions = computed(() => sessions.value.filter((session) => session.username).length);
const selectedUserRecord = computed(() => users.value.find((user) => user.username === selectedUser.value) ?? null);
const isHostAdmin = computed(() => hostStatus.value?.is_windows === true && hostStatus.value?.is_admin_account === true);
const isElevated = computed(() => hostStatus.value?.is_elevated === true);
const isLocked = computed(() => hostStatus.value?.is_windows === true && hostStatus.value?.is_admin_account === false);
const installServerUrl = computed(() => `http://${installBindHost.value}:${installPort.value}`);
const currentPage = computed(() => navigation.find((item) => item.id === activeTab.value) ?? navigation[0]);
const canLoadRemoteData = computed(() => serverUrl.value.trim().length > 0 && adminToken.value.trim().length > 0);
const canReset = computed(() => selectedUser.value.length > 0 && newPassword.value.length >= 12 && adminToken.value.length > 0);
const canCreate = computed(() => createUsername.value.length > 0 && createPassword.value.length >= 12 && adminToken.value.length > 0);
const canDelete = computed(() => selectedUser.value.length > 0 && deleteConfirmation.value === selectedUser.value && adminToken.value.length > 0);
const canAddServer = computed(() => newServerName.value.trim().length > 0 && newServerHost.value.trim().length > 0 && Number(newServerPort.value) > 0);
const installDisabledReason = computed(() => {
  if (hostStatusLoading.value) return 'Checking local Windows privileges.';
  if (hostStatus.value?.is_windows !== true) return 'Server installation is available only on Windows.';
  if (!isHostAdmin.value) return 'Sign in with a local administrator account.';
  if (!isElevated.value) return 'Run WinPassageAdmin as administrator to install the service.';
  if (!installBindHost.value.trim()) return 'Set a bind host first.';
  if (installAdminToken.value.trim().length > 0 && installAdminToken.value.trim().length < 16) return 'Use an admin token with at least 16 characters, or leave it empty to generate one.';
  const port = Number(installPort.value);
  if (!Number.isInteger(port) || port < 1 || port > 65535) return 'Choose a valid TCP port between 1 and 65535.';
  return '';
});
const canInstallServer = computed(() => installDisabledReason.value === '');
const canDemoteServer = computed(() => isElevated.value && demoteConfirmation.value === 'REMOVE SERVER');

function applyTheme() {
  const resolved = theme.value === 'system' ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light') : theme.value;
  document.documentElement.dataset.theme = resolved;
  localStorage.setItem(THEME_KEY, theme.value);
}

function setTheme(value: ThemeMode) {
  theme.value = value;
  applyTheme();
}

function persistProfiles() {
  localStorage.setItem(PROFILE_STORAGE_KEY, JSON.stringify(serverProfiles.value));
  localStorage.setItem(ACTIVE_PROFILE_KEY, activeServerId.value);
}

function syncActiveProfileFromServerUrl() {
  const profile = activeServerProfile.value;
  if (!profile) return;

  try {
    const url = normalizeUrl(serverUrl.value);
    profile.protocol = url.protocol === 'https:' ? 'https' : 'http';
    profile.host = url.hostname;
    profile.port = Number(url.port || (profile.protocol === 'https' ? 443 : 80));
    serverUrl.value = profileUrl(profile);
  } catch {
    error.value = 'Use a server address such as http://192.168.1.10:4487.';
  }
}

function persistSettings() {
  syncActiveProfileFromServerUrl();
  persistProfiles();
}

function selectServerProfile(profile: ServerProfile) {
  activeServerId.value = profile.id;
  serverUrl.value = profileUrl(profile);
  users.value = [];
  sessions.value = [];
  selectedUser.value = '';
  persistProfiles();
}

function addServerProfile() {
  if (!canAddServer.value) return;

  const profile: ServerProfile = {
    id: crypto.randomUUID(),
    name: newServerName.value.trim(),
    network_name: newServerNetwork.value.trim() || 'Unspecified network',
    protocol: newServerProtocol.value,
    host: newServerHost.value.trim(),
    port: Number(newServerPort.value),
    notes: newServerNotes.value.trim() || null,
  };

  serverProfiles.value.push(profile);
  newServerName.value = '';
  newServerNetwork.value = '';
  newServerHost.value = '';
  newServerPort.value = 4487;
  newServerProtocol.value = 'http';
  newServerNotes.value = '';
  selectServerProfile(profile);
  message.value = `Added ${profile.name}.`;
}

function removeServerProfile(profile: ServerProfile) {
  if (serverProfiles.value.length <= 1) {
    error.value = 'At least one server profile must remain.';
    return;
  }

  serverProfiles.value = serverProfiles.value.filter((item) => item.id !== profile.id);
  if (activeServerId.value === profile.id) selectServerProfile(serverProfiles.value[0]);
  persistProfiles();
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
  if (!response.ok) throw new Error(payload.error ?? `Request failed with ${response.status}`);
  return payload as T;
}

async function runTask(task: () => Promise<string>, refresh = true) {
  loading.value = true;
  message.value = '';
  error.value = '';
  try {
    message.value = await task();
    if (refresh) await Promise.allSettled([loadUsers(false), loadSessions(false)]);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    loading.value = false;
  }
}

async function loadHostStatus() {
  hostStatusLoading.value = true;
  try {
    hostStatus.value = await invoke<AdminHostStatus>('get_admin_host_status');
    installSourceDir.value = hostStatus.value.executable_dir ?? installSourceDir.value;
    installDir.value = hostStatus.value.install_dir || installDir.value;
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    hostStatusLoading.value = false;
  }
}

async function loadUsers(showMessage = true) {
  const payload = await request<ListUsersResponse>('/v1/users');
  users.value = payload.users;
  if (!selectedUser.value && payload.users[0]) selectedUser.value = payload.users[0].username;
  if (showMessage) message.value = `Loaded ${payload.users.length} local users.`;
}

async function loadSessions(showMessage = true) {
  const payload = await request<ListSessionsResponse>('/v1/sessions');
  sessions.value = payload.sessions;
  if (showMessage) message.value = `Loaded ${payload.sessions.length} sessions.`;
}

async function refreshAll() {
  await runTask(async () => {
    await Promise.all([loadUsers(false), loadSessions(false)]);
    return 'Users and sessions refreshed.';
  }, false);
}

async function createUser() {
  await runTask(async () => {
    const payload = await request<ActionResponse>('/v1/users', {
      method: 'POST',
      body: JSON.stringify({
        username: createUsername.value.trim(),
        full_name: createFullName.value.trim() || null,
        password: createPassword.value,
        must_change_password: createMustChange.value,
        enabled: createEnabled.value,
        admin: createAdmin.value,
        reason: reason.value || null,
        request_id: crypto.randomUUID(),
      }),
    });
    selectedUser.value = createUsername.value.trim();
    createUsername.value = '';
    createFullName.value = '';
    createPassword.value = '';
    return payload.message;
  });
}

async function resetPassword() {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/users/${encodeURIComponent(selectedUser.value)}/password/reset`, {
      method: 'POST',
      body: JSON.stringify({ new_password: newPassword.value, reason: reason.value || null, request_id: crypto.randomUUID() }),
    });
    newPassword.value = '';
    return payload.message;
  });
}

async function setEnabled(enabled: boolean) {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/users/${encodeURIComponent(selectedUser.value)}/enabled`, {
      method: 'POST',
      body: JSON.stringify({ enabled, reason: reason.value || null, request_id: crypto.randomUUID() }),
    });
    return payload.message;
  });
}

async function setAdministrator(enabled: boolean) {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/users/${encodeURIComponent(selectedUser.value)}/admin`, {
      method: 'POST',
      body: JSON.stringify({ enabled, reason: reason.value || null, request_id: crypto.randomUUID() }),
    });
    return payload.message;
  });
}

async function deleteUser() {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/users/${encodeURIComponent(selectedUser.value)}`, {
      method: 'DELETE',
      body: JSON.stringify({
        confirmation: deleteConfirmation.value,
        logoff_sessions: deleteLogoffSessions.value,
        delete_profile: deleteProfile.value,
        reason: reason.value || null,
        request_id: crypto.randomUUID(),
      }),
    });
    selectedUser.value = '';
    deleteConfirmation.value = '';
    return payload.message;
  });
}

async function logoffSession(session: LocalSessionSummary) {
  await runTask(async () => {
    const payload = await request<ActionResponse>(`/v1/sessions/${session.session_id}/logoff`, {
      method: 'POST',
      body: JSON.stringify({ reason: reason.value || null, request_id: crypto.randomUUID() }),
    });
    return payload.message;
  });
}

async function installServer() {
  if (!installAdminToken.value.trim()) {
    installAdminToken.value = crypto.randomUUID().replaceAll('-', '') + crypto.randomUUID().slice(0, 8);
  }

  await runTask(async () => {
    const payload = await invoke<ServerInstallResult>('install_server_mode', {
      request: {
        source_dir: installSourceDir.value || null,
        install_dir: installDir.value || null,
        bind_host: installBindHost.value,
        port: Number(installPort.value),
        admin_token: installAdminToken.value,
        require_tls: installRequireTls.value,
      },
    });
    await loadHostStatus();
    return payload.message;
  }, false);
}

async function demoteServer() {
  await runTask(async () => {
    const payload = await invoke<ServerInstallResult>('demote_server_mode', {
      request: {
        install_dir: installDir.value || null,
        confirmation: 'DEMOTE SERVER',
        remove_files: demoteRemoveFiles.value,
      },
    });
    demoteConfirmation.value = '';
    await loadHostStatus();
    return payload.message;
  }, false);
}

function finishInitialSetup() {
  persistSettings();
  localStorage.setItem(SETUP_KEY, 'true');
  initialSetupOpen.value = false;
  void nextTick(() => startAdminTour());
}

function startAdminTour(force = false) {
  if (!force && tourCompleted.value) return;
  const tour = driver({
    showProgress: true,
    allowClose: true,
    nextBtnText: 'Next',
    prevBtnText: 'Back',
    doneBtnText: 'Done',
    steps: [
      { element: '#admin-server', popover: { title: 'Server profile', description: 'Select the central Windows Pro machine by IP address and port.' } },
      { element: '#admin-users', popover: { title: 'Windows users', description: 'Users are read from Windows directly, not from a WinPassage database.' } },
      { element: '#admin-install', popover: { title: 'Server install', description: 'Installing or removing the service requires elevation.' } },
      { element: '#admin-settings', popover: { title: 'Settings', description: 'Choose the app theme and review connection defaults.' } },
    ],
    onDestroyed: () => {
      tourCompleted.value = true;
      localStorage.setItem(TOUR_KEY, 'true');
    },
  });
  tour.drive();
}

onMounted(() => {
  applyTheme();
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', applyTheme);
  void loadHostStatus();
  if (!initialSetupOpen.value) void nextTick(() => startAdminTour());
});
</script>

<template>
  <main class="app app-admin">
    <section v-if="initialSetupOpen" class="setup-overlay" role="dialog" aria-modal="true">
      <article class="setup-card">
        <div class="setup-brand" v-html="brandIcon"></div>
        <p class="eyebrow">Initial setup</p>
        <h1>Prepare WinPassageAdmin</h1>
        <p class="lead">Set the server address and token. The guided tour starts after setup.</p>
        <div class="form-grid two">
          <label>Server address<input v-model="serverUrl" placeholder="http://192.168.1.10:4487" /></label>
          <label>Admin token<input v-model="adminToken" type="password" autocomplete="off" placeholder="Admin token" /></label>
        </div>
        <div class="button-row end">
          <button class="btn-primary" :disabled="!serverUrl.trim()" @click="finishInitialSetup">Save and continue</button>
        </div>
      </article>
    </section>

    <section class="app-shell">
      <aside class="sidebar" aria-label="WinPassageAdmin navigation">
        <div class="brand-row">
          <span class="app-mark" v-html="brandIcon"></span>
          <div>
            <p class="brand-title">WinPassageAdmin</p>
            <p class="brand-subtitle">Windows Pro bridge</p>
          </div>
        </div>

        <nav class="nav-list" aria-label="Main sections">
          <button v-for="item in navigation" :key="item.id" class="nav-item" :class="{ active: activeTab === item.id }" @click="activeTab = item.id">
            <span v-html="icon(item.icon)"></span>{{ item.label }}
          </button>
        </nav>

        <div class="sidebar-status">
          <p class="status-line"><span class="dot success"></span>{{ isElevated ? 'Elevated' : isHostAdmin ? 'Admin account' : 'Standard user' }}</p>
          <p>{{ activeServerProfile?.host }}:{{ activeServerProfile?.port }}</p>
        </div>
      </aside>

      <section class="workspace">
        <header class="topbar">
          <div>
            <p class="eyebrow">{{ currentPage.label }}</p>
            <h1>{{ currentPage.label }}</h1>
          </div>
          <div class="topbar-actions">
            <button class="btn-ghost" @click="initialSetupOpen = true"><span v-html="icon('settings')"></span>Setup</button>
            <button class="btn-ghost" @click="startAdminTour(true)"><span v-html="icon('play')"></span>Guide</button>
            <button class="btn-primary" :disabled="loading || !canLoadRemoteData" @click="refreshAll"><span v-html="icon('refresh')"></span>Refresh</button>
          </div>
        </header>

        <section v-if="isLocked" class="lock-panel">
          <span class="lock-mark" v-html="brandIcon"></span>
          <h2>Administrator account required</h2>
          <p>Sign in with a Windows local administrator account to use WinPassageAdmin.</p>
          <p class="muted">{{ hostStatus?.message || 'Checking account privileges…' }}</p>
        </section>

        <template v-else>
          <p v-if="message" class="notice success"><span v-html="icon('check')"></span>{{ message }}</p>
          <p v-if="error" class="notice error"><span v-html="icon('warning')"></span>{{ error }}</p>

          <section v-if="activeTab === 'overview'" class="page-stack">
            <article v-if="hostStatus && !isElevated" class="notice info">
              <span v-html="icon('info')"></span>{{ hostStatus.message }}
            </article>

            <div class="page-grid two">
              <article class="surface-card" id="admin-install">
                <div class="card-title-row">
                  <div>
                    <h2>Server on this computer</h2>
                    <p>Install the WinPassage service locally with one action.</p>
                  </div>
                  <span class="hero-mark" v-html="brandIcon"></span>
                </div>
                <div class="stat-list">
                  <span>Status</span><strong>{{ hostStatus?.service_installed ? 'Installed' : 'Not installed' }}</strong>
                  <span>Address</span><strong>{{ installBindHost }}:{{ installPort }}</strong>
                  <span>Folder</span><strong>{{ installDir }}</strong>
                </div>
                <p v-if="installDisabledReason" class="notice info"><span v-html="icon('info')"></span>{{ installDisabledReason }}</p>
                <button class="btn-primary full" :disabled="loading || !canInstallServer" @click="installServer">Install server on this computer</button>
              </article>

              <article class="surface-card" id="admin-server">
                <div class="card-title-row">
                  <div><h2>Active profile</h2><p>Connection shortcut for the selected central machine.</p></div>
                </div>
                <div class="stat-list">
                  <span>Name</span><strong>{{ activeServerProfile?.name }}</strong>
                  <span>Endpoint</span><strong>{{ serverUrl }}</strong>
                  <span>Loaded users</span><strong>{{ users.length }}</strong>
                  <span>Sessions</span><strong>{{ sessions.length }}</strong>
                </div>
              </article>
            </div>

            <article class="surface-card guide-card">
              <div>
                <h2>Initial setup and guided tour</h2>
                <p class="muted">Finish setup first, then use the tour when you need orientation.</p>
              </div>
              <div class="button-row">
                <button class="btn-secondary" @click="initialSetupOpen = true">Initial setup</button>
                <button class="btn-secondary" @click="startAdminTour(true)"><span v-html="icon('play')"></span>Start tour</button>
              </div>
            </article>
          </section>

          <section v-if="activeTab === 'users'" id="admin-users" class="page-grid two-wide">
            <article class="surface-card table-card">
              <div class="card-title-row">
                <div><h2>Windows users</h2><p>Read directly from this Windows machine.</p></div>
                <span class="pill neutral">{{ users.length }} users</span>
              </div>
              <div class="table-wrap">
                <table class="data-table">
                  <thead><tr><th>User</th><th>Name</th><th>Status</th><th>Access</th><th>Sessions</th></tr></thead>
                  <tbody>
                    <tr v-for="user in users" :key="user.username" :class="{ selected: selectedUser === user.username }" @click="selectedUser = user.username">
                      <td><strong>{{ user.username }}</strong></td>
                      <td>{{ user.full_name || '—' }}</td>
                      <td><span class="pill" :class="user.disabled ? 'danger' : 'success'">{{ user.disabled ? 'Disabled' : 'Enabled' }}</span></td>
                      <td><span class="pill" :class="user.is_administrator ? 'warning' : 'neutral'">{{ user.is_administrator ? 'Admin' : 'Standard' }}</span></td>
                      <td>{{ user.active_session_count }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div v-if="users.length === 0" class="empty-state"><span v-html="brandIcon"></span><h3>No users loaded</h3><p>Refresh after entering the admin token.</p></div>
            </article>

            <aside class="side-panel">
              <article class="surface-card">
                <h2>Create user</h2>
                <div class="form-grid">
                  <label>Username<input v-model="createUsername" placeholder="julia" /></label>
                  <label>Initial password<input v-model="createPassword" type="password" autocomplete="new-password" /></label>
                </div>
                <details class="advanced">
                  <summary>More options</summary>
                  <div class="form-grid">
                    <label>Full name<input v-model="createFullName" placeholder="Julia Nagy" /></label>
                    <label class="check-row"><input v-model="createMustChange" type="checkbox" />Require change at next sign-in</label>
                    <label class="check-row"><input v-model="createEnabled" type="checkbox" />Enable account</label>
                    <label class="check-row"><input v-model="createAdmin" type="checkbox" />Add to Administrators</label>
                  </div>
                </details>
                <button class="btn-primary full" :disabled="loading || !canCreate" @click="createUser">Create user</button>
              </article>

              <article class="surface-card">
                <h2>Selected user</h2>
                <p class="muted">{{ selectedUserRecord?.username || 'No user selected' }}</p>
                <label>New password<input v-model="newPassword" type="password" autocomplete="new-password" /></label>
                <label>Audit reason<textarea v-model="reason" rows="2" placeholder="Reason"></textarea></label>
                <div class="button-grid">
                  <button class="btn-primary" :disabled="loading || !canReset" @click="resetPassword">Reset</button>
                  <button class="btn-secondary" :disabled="loading || !selectedUser" @click="setEnabled(true)">Enable</button>
                  <button class="btn-secondary" :disabled="loading || !selectedUser" @click="setEnabled(false)">Disable</button>
                  <button class="btn-secondary" :disabled="loading || !selectedUser" @click="setAdministrator(true)">Grant admin</button>
                </div>
                <details class="advanced">
                  <summary>Danger zone</summary>
                  <div class="form-grid">
                    <button class="btn-danger" :disabled="loading || !selectedUser" @click="setAdministrator(false)">Revoke admin</button>
                    <label>Type username to delete<input v-model="deleteConfirmation" :placeholder="selectedUser || 'username'" /></label>
                    <label class="check-row"><input v-model="deleteLogoffSessions" type="checkbox" />Log off sessions first</label>
                    <label class="check-row"><input v-model="deleteProfile" type="checkbox" />Request profile deletion</label>
                    <button class="btn-danger full" :disabled="loading || !canDelete" @click="deleteUser">Delete account</button>
                  </div>
                </details>
              </article>
            </aside>
          </section>

          <section v-if="activeTab === 'sessions'" class="page-stack">
            <article class="surface-card table-card">
              <div class="card-title-row"><div><h2>Sessions</h2><p>Log off active Windows sessions when required.</p></div><span class="pill neutral">{{ sessions.length }} sessions</span></div>
              <div class="table-wrap">
                <table class="data-table">
                  <thead><tr><th>ID</th><th>User</th><th>State</th><th>Client</th><th></th></tr></thead>
                  <tbody>
                    <tr v-for="session in sessions" :key="session.session_id">
                      <td>{{ session.session_id }}</td><td>{{ session.username || '—' }}</td><td>{{ session.state }}</td><td>{{ session.client_name || (session.is_console ? 'Console' : '—') }}</td>
                      <td><button class="btn-secondary small" :disabled="loading" @click="logoffSession(session)">Log off</button></td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </article>
          </section>

          <section v-if="activeTab === 'servers'" class="page-grid two-wide">
            <article class="surface-card">
              <div class="card-title-row"><div><h2>Server profiles</h2><p>IP:port shortcuts for standalone Windows Pro machines.</p></div></div>
              <div class="profile-list">
                <button v-for="profile in serverProfiles" :key="profile.id" class="profile-row" :class="{ selected: activeServerId === profile.id }" @click="selectServerProfile(profile)">
                  <span><strong>{{ profile.name }}</strong><small>{{ profile.network_name }} · {{ profile.host }}:{{ profile.port }}</small></span>
                  <span v-html="icon('chevronRight')"></span>
                </button>
              </div>
            </article>

            <aside class="side-panel">
              <article class="surface-card">
                <h2>Add profile</h2>
                <div class="form-grid">
                  <label>Name<input v-model="newServerName" placeholder="Front office" /></label>
                  <label>Host<input v-model="newServerHost" placeholder="192.168.1.10" /></label>
                  <label>Port<input v-model.number="newServerPort" type="number" min="1" max="65535" /></label>
                </div>
                <details class="advanced">
                  <summary>More options</summary>
                  <div class="form-grid">
                    <label>Network<input v-model="newServerNetwork" placeholder="Budapest" /></label>
                    <label>Protocol<select v-model="newServerProtocol"><option value="http">http</option><option value="https">https</option></select></label>
                    <label>Notes<input v-model="newServerNotes" placeholder="Optional" /></label>
                  </div>
                </details>
                <div class="button-row"><button class="btn-primary" :disabled="!canAddServer" @click="addServerProfile">Add profile</button><button class="btn-ghost" :disabled="!activeServerProfile" @click="activeServerProfile && removeServerProfile(activeServerProfile)">Remove selected</button></div>
              </article>
            </aside>
          </section>

          <section v-if="activeTab === 'settings'" id="admin-settings" class="page-grid two">
            <article class="surface-card">
              <h2>Appearance</h2>
              <p class="muted">Choose the interface theme.</p>
              <div class="segmented">
                <button :class="{ active: theme === 'light' }" @click="setTheme('light')"><span v-html="icon('sun')"></span>Light</button>
                <button :class="{ active: theme === 'system' }" @click="setTheme('system')"><span v-html="icon('monitor')"></span>System</button>
                <button :class="{ active: theme === 'dark' }" @click="setTheme('dark')"><span v-html="icon('moon')"></span>Dark</button>
              </div>
            </article>

            <article class="surface-card">
              <h2>Server install settings</h2>
              <p class="muted">Used by the one-click local install action.</p>
              <div class="form-grid two">
                <label>Bind host<input v-model="installBindHost" /></label>
                <label>Port<input v-model.number="installPort" type="number" min="1" max="65535" /></label>
                <label>Install folder<input v-model="installDir" /></label>
                <label>Source folder<input v-model="installSourceDir" /></label>
              </div>
              <label class="check-row"><input v-model="installRequireTls" type="checkbox" />Require TLS boundary</label>
            </article>

            <article class="surface-card">
              <h2>Guided tour</h2>
              <p class="muted">Replay onboarding at any time.</p>
              <button class="btn-secondary" @click="startAdminTour(true)">Replay tour</button>
            </article>

            <article class="surface-card">
              <h2>Advanced</h2>
              <details class="advanced">
                <summary>Remove local server service</summary>
                <div class="form-grid">
                  <p class="muted">Stops and removes the WinPassage service from this computer. It does not delete Windows users.</p>
                  <label>Type REMOVE SERVER<input v-model="demoteConfirmation" placeholder="REMOVE SERVER" /></label>
                  <label class="check-row"><input v-model="demoteRemoveFiles" type="checkbox" />Remove copied executables</label>
                  <button class="btn-danger full" :disabled="loading || !canDemoteServer" @click="demoteServer">Remove server service</button>
                </div>
              </details>
            </article>
          </section>
        </template>
      </section>
    </section>
  </main>
</template>
