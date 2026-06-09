<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, nextTick, onMounted, ref } from 'vue';
import { driver } from 'driver.js';
import 'driver.js/dist/driver.css';
import type {
  ActionResponse,
  ListSessionsResponse,
  ListUsersResponse,
  LocalSessionSummary,
  LocalUserSummary,
  ServerInstallResult,
  AdminHostStatus,
  ServerProfile,
} from './types';

const PROFILE_STORAGE_KEY = 'winpassage.admin.serverProfiles';
const ACTIVE_PROFILE_KEY = 'winpassage.admin.activeServerId';

function defaultServerProfiles(): ServerProfile[] {
  return [
    {
      id: 'local-dev',
      name: 'Local development',
      network_name: 'Local machine',
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

const serverProfiles = ref<ServerProfile[]>(loadServerProfiles());
const activeServerId = ref(localStorage.getItem(ACTIVE_PROFILE_KEY) ?? serverProfiles.value[0]?.id ?? 'local-dev');
const activeServerProfile = computed(() => serverProfiles.value.find((profile) => profile.id === activeServerId.value) ?? serverProfiles.value[0]);
const serverUrl = ref(activeServerProfile.value ? profileUrl(activeServerProfile.value) : 'http://localhost:4487');
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

const enabledUsers = computed(() => users.value.filter((user) => !user.disabled).length);
const adminUsers = computed(() => users.value.filter((user) => user.is_administrator).length);
const activeSessions = computed(() => sessions.value.filter((session) => session.username).length);
const selectedUserRecord = computed(() => users.value.find((user) => user.username === selectedUser.value) ?? null);
const passwordLengthHint = computed(() => `${newPassword.value.length}/12 minimum characters`);
const canReset = computed(() => selectedUser.value.length > 0 && newPassword.value.length >= 12 && adminToken.value.length > 0);
const canCreate = computed(() => createUsername.value.length > 0 && createPassword.value.length >= 12 && adminToken.value.length > 0);
const canDelete = computed(() => selectedUser.value.length > 0 && deleteConfirmation.value === selectedUser.value && adminToken.value.length > 0);
const canAddServer = computed(() => newServerName.value.trim().length > 0 && newServerHost.value.trim().length > 0 && Number(newServerPort.value) > 0);
const isHostAdmin = computed(() => hostStatus.value?.is_windows === true && hostStatus.value?.is_admin_account === true);
const isElevated = computed(() => hostStatus.value?.is_elevated === true);
const installServerUrl = computed(() => `http://${installBindHost.value}:${installPort.value}`);
const canInstallServer = computed(
  () =>
    isElevated.value &&
    installBindHost.value.trim().length > 0 &&
    Number(installPort.value) > 0 &&
    Number(installPort.value) <= 65535 &&
    installAdminToken.value.trim().length >= 16,
);
const canDemoteServer = computed(() => isElevated.value && demoteConfirmation.value === 'DEMOTE SERVER');
const initialSetupOpen = ref(localStorage.getItem('winpassage.admin.initialSetupComplete') !== 'true');
const tourCompleted = ref(localStorage.getItem('winpassage.admin.tourComplete') === 'true');

async function loadHostStatus() {
  hostStatusLoading.value = true;
  try {
    hostStatus.value = await invoke<AdminHostStatus>('get_admin_host_status');
    if (hostStatus.value.executable_dir && !installSourceDir.value) {
      installSourceDir.value = hostStatus.value.executable_dir;
    }
    installDir.value = hostStatus.value.install_dir || installDir.value;
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    hostStatusLoading.value = false;
  }
}

function persistProfiles() {
  localStorage.setItem(PROFILE_STORAGE_KEY, JSON.stringify(serverProfiles.value));
  localStorage.setItem(ACTIVE_PROFILE_KEY, activeServerId.value);
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
  message.value = `Added server profile ${profile.name}.`;
}

function removeServerProfile(profile: ServerProfile) {
  if (serverProfiles.value.length <= 1) {
    error.value = 'At least one server profile must remain.';
    return;
  }

  serverProfiles.value = serverProfiles.value.filter((item) => item.id !== profile.id);
  if (activeServerId.value === profile.id) {
    selectServerProfile(serverProfiles.value[0]);
  }
  persistProfiles();
  message.value = `Removed server profile ${profile.name}.`;
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
    error.value = 'Invalid server URL. Use an address such as http://192.168.1.10:4487.';
  }
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

async function runLocalTask(task: () => Promise<string>) {
  loading.value = true;
  error.value = '';
  message.value = '';

  try {
    message.value = await task();
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    loading.value = false;
  }
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
    message.value = `Loaded ${payload.users.length} local Windows accounts from ${activeServerProfile.value?.name ?? serverUrl.value}.`;
  }
}

async function loadSessions(showMessage = true) {
  const payload = await request<ListSessionsResponse>('/v1/sessions');
  sessions.value = payload.sessions;
  if (showMessage) {
    message.value = `Loaded ${payload.sessions.length} Windows sessions from ${activeServerProfile.value?.name ?? serverUrl.value}.`;
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
    return `Loaded ${userPayload.users.length} users and ${sessionPayload.sessions.length} sessions from ${activeServerProfile.value?.name ?? serverUrl.value}.`;
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

async function installServerMode() {
  await runLocalTask(async () => {
    const result = await invoke<ServerInstallResult>('install_server_mode', {
      request: {
        source_dir: installSourceDir.value || null,
        install_dir: installDir.value || null,
        bind_host: installBindHost.value,
        port: Number(installPort.value),
        admin_token: installAdminToken.value,
        require_tls: installRequireTls.value,
      },
    });

    serverUrl.value = result.server_url || installServerUrl.value;
    adminToken.value = installAdminToken.value;
    return `${result.message} Use ${serverUrl.value} from clients and admin profiles.`;
  });
}

async function demoteServerMode() {
  await runLocalTask(async () => {
    const result = await invoke<ServerInstallResult>('demote_server_mode', {
      request: {
        install_dir: installDir.value || null,
        confirmation: demoteConfirmation.value,
        remove_files: demoteRemoveFiles.value,
      },
    });

    demoteConfirmation.value = '';
    return result.message;
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


function saveInitialSetup() {
  localStorage.setItem('winpassage.admin.initialSetupComplete', 'true');
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
      {
        element: '#admin-server-install',
        popover: {
          title: 'Install the server',
          description: 'Use this section only on the Windows Pro machine that should run the WinPassage service.',
        },
      },
      {
        element: '#admin-server-registry',
        popover: {
          title: 'Add servers by IP and port',
          description: 'Each profile points to one standalone WinPassage server. Profiles do not copy users between machines.',
        },
      },
      {
        element: '#admin-users',
        popover: {
          title: 'Manage real Windows accounts',
          description: 'Users and sessions are read from the selected server. WinPassage does not keep a separate user database.',
        },
      },
    ],
    onDestroyed: () => {
      tourCompleted.value = true;
      localStorage.setItem('winpassage.admin.tourComplete', 'true');
    },
  });

  tour.drive();
}

onMounted(async () => {
  await loadHostStatus();
  if (!initialSetupOpen.value) {
    void nextTick(() => startAdminTour());
  }
});

</script>

<template>
  <main class="app app-admin">
    <section v-if="initialSetupOpen && isHostAdmin" class="setup-overlay" role="dialog" aria-modal="true">
      <article class="setup-card">
        <p class="eyebrow">First run</p>
        <h1>Set the first admin connection.</h1>
        <p class="lead">Confirm the local service port and admin token before the guide starts.</p>
        <div class="grid two">
          <label>
            Server port
            <input v-model.number="installPort" type="number" min="1" max="65535" />
          </label>
          <label>
            Admin token
            <input v-model="installAdminToken" type="password" autocomplete="new-password" placeholder="At least 16 characters" />
          </label>
        </div>
        <div class="actions">
          <button :disabled="installAdminToken.trim().length < 16" @click="saveInitialSetup">Save and show guide</button>
          <button class="secondary" @click="initialSetupOpen = false; startAdminTour(true)">Skip setup</button>
        </div>
      </article>
    </section>
    <section v-if="hostStatusLoading" class="lock-screen">
      <article class="lock-card">
        <div class="lock-icon">…</div>
        <p class="eyebrow">Checking privileges</p>
        <h1>Preparing the admin console.</h1>
        <p class="lead">WinPassage is checking whether this Windows account can manage the local server installation.</p>
      </article>
    </section>

    <section v-else-if="!isHostAdmin" class="lock-screen">
      <article class="lock-card">
        <div class="lock-icon">🔒</div>
        <p class="eyebrow">Administrator account required</p>
        <h1>This console is locked.</h1>
        <p class="lead">{{ hostStatus?.message }}</p>
        <ol class="step-list">
          <li class="step-item"><span class="step-number">1</span><span>Sign out from the standard Windows account.</span></li>
          <li class="step-item"><span class="step-number">2</span><span>Sign in with a local administrator account on this computer.</span></li>
          <li class="step-item"><span class="step-number">3</span><span>Open WinPassageAdmin again. Use Run as administrator when installing or removing the service.</span></li>
        </ol>
        <div class="actions">
          <button class="secondary" @click="loadHostStatus">Check again</button>
        </div>
      </article>
    </section>

    <section v-else class="app-shell">
      <aside class="side-rail" aria-label="WinPassageAdmin navigation">
        <div class="brand-block">
          <div class="brand-mark" aria-hidden="true"><span class="bridge-icon">⊞</span></div>
          <div>
            <p class="brand-title">WinPassage</p>
            <p class="brand-subtitle">Admin console</p>
          </div>
        </div>

        <nav class="rail-menu" aria-label="Workspace sections">
          <span class="rail-item active"><span class="rail-dot"></span>Overview</span>
          <span class="rail-item">Networks</span>
          <span class="rail-item">Server install</span>
          <span class="rail-item">Users</span>
          <span class="rail-item">Access</span>
          <span class="rail-item">Sessions</span>
          <span class="rail-item">Audit</span>
        </nav>

        <div class="rail-footer">
          <p class="metric-label">Security note</p>
          <p class="muted">Server profiles are local admin-console settings. Windows users are always read from the selected server.</p>
        </div>
      </aside>

      <section class="workspace">
        <header class="hero-panel">
          <div class="hero-grid">
            <div>
              <p class="eyebrow">Windows Pro central machines</p>
              <h1>Manage local Windows Pro servers.</h1>
              <p class="lead">
                Add each server by IP and port. User operations always run on the selected Windows machine.
              </p>
            </div>
            <div class="status-stack">
              <span class="status-pill success">Windows accounts</span>
              <span class="status-pill warning">Per-server actions</span>
              <span class="status-pill">Small networks</span>
            </div>
          </div>
        </header>

        <section class="metric-grid" aria-label="Server and user statistics">
          <article class="metric-card">
            <p class="metric-label">Server profiles</p>
            <p class="metric-value">{{ serverProfiles.length }}</p>
            <p class="metric-hint">Separate central machines</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Active server</p>
            <p class="metric-value compact">{{ activeServerProfile?.name || '—' }}</p>
            <p class="metric-hint">{{ serverUrl }}</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Loaded users</p>
            <p class="metric-value">{{ users.length }}</p>
            <p class="metric-hint">Current Windows accounts</p>
          </article>
          <article class="metric-card">
            <p class="metric-label">Sessions</p>
            <p class="metric-value">{{ activeSessions }}</p>
            <p class="metric-hint">Named Windows sessions</p>
          </article>
        </section>

        <section id="admin-server-install" class="surface-card server-install-card">
          <div class="card-heading">
            <div>
              <h2>Make this computer a WinPassage server</h2>
              <p class="muted">Install the background service on this Windows Pro machine. Clients and admins connect later with an <strong>IP:port</strong> address.</p>
            </div>
            <span class="status-pill success">Administrator verified</span>
          </div>

          <p v-if="!isElevated" class="notice warning">You are signed in with an administrator account, but this app is not elevated. Close it and choose <strong>Run as administrator</strong> before installing or removing the service.</p>

          <p class="notice warning">Place <strong>winpassage-server.exe</strong>, <strong>winpassage-agentctl.exe</strong>, and <strong>winpassage-updater.exe</strong> next to WinPassageAdmin or choose their source folder before installing.</p>

          <div class="grid three">
            <label>
              Binary source folder
              <input v-model="installSourceDir" placeholder="Folder containing the service executables" />
            </label>
            <label>
              Install folder
              <input v-model="installDir" placeholder="C:\Program Files\WinPassage" />
            </label>
            <label>
              Admin token for this server
              <input v-model="installAdminToken" type="password" autocomplete="new-password" placeholder="At least 16 characters" />
            </label>
            <label>
              Bind host
              <input v-model="installBindHost" placeholder="0.0.0.0" />
            </label>
            <label>
              Port
              <input v-model.number="installPort" type="number" min="1" max="65535" />
            </label>
            <label class="toggle-row install-toggle"><input v-model="installRequireTls" type="checkbox" /> Require TLS at the service boundary</label>
          </div>

          <div class="selected-card">
            <p class="metric-label">Client/admin connection address</p>
            <strong>{{ installServerUrl }}</strong>
            <span class="muted">Use the server computer's LAN IP address instead of 0.0.0.0 when adding client or admin profiles.</span>
          </div>

          <div class="actions">
            <button :disabled="loading || !canInstallServer" @click="installServerMode">Install server service</button>
            <button class="secondary" :disabled="loading" @click="loadHostStatus">Recheck admin rights</button>
          </div>
        </section>

        <section class="danger-card">
          <div class="card-heading">
            <div>
              <h2>Demote this computer back to client-only mode</h2>
              <p class="muted">Stops and removes the local WinPassage service. This does not delete Windows users or profiles.</p>
            </div>
            <span class="status-pill danger">Destructive</span>
          </div>
          <div class="grid two">
            <label>
              Type DEMOTE SERVER to confirm
              <input v-model="demoteConfirmation" placeholder="DEMOTE SERVER" />
            </label>
            <label class="toggle-row install-toggle"><input v-model="demoteRemoveFiles" type="checkbox" /> Remove installed WinPassage service executables</label>
          </div>
          <div class="actions">
            <button class="danger" :disabled="loading || !canDemoteServer" @click="demoteServerMode">Remove server service</button>
          </div>
        </section>

        <section id="admin-server-registry" class="surface-card">
          <div class="card-heading">
            <div>
              <h2>Server registry</h2>
              <p class="muted">Add every standalone WinPassage server machine by IP address or stable DNS name. These profiles are only admin-console shortcuts.</p>
            </div>
            <span class="status-pill">{{ activeServerProfile?.network_name || 'No network' }}</span>
          </div>

          <div class="server-grid">
            <article
              v-for="profile in serverProfiles"
              :key="profile.id"
              class="server-card"
              :class="{ active: profile.id === activeServerId }"
            >
              <div>
                <p class="metric-label">{{ profile.network_name }}</p>
                <h3>{{ profile.name }}</h3>
                <p class="server-address">{{ profileUrl(profile) }}</p>
                <p class="muted">{{ profile.notes || 'Standalone Windows Pro server profile.' }}</p>
              </div>
              <div class="actions wrap">
                <button class="secondary small" :disabled="profile.id === activeServerId" @click="selectServerProfile(profile)">Use</button>
                <button class="danger small" :disabled="serverProfiles.length <= 1" @click="removeServerProfile(profile)">Remove</button>
              </div>
            </article>
          </div>

          <div class="grid three server-form">
            <label>
              Name
              <input v-model="newServerName" placeholder="Office server" />
            </label>
            <label>
              Network
              <input v-model="newServerNetwork" placeholder="Budapest office" />
            </label>
            <label>
              IP address or DNS name
              <input v-model="newServerHost" placeholder="192.168.1.10" />
            </label>
            <label>
              Protocol
              <select v-model="newServerProtocol">
                <option value="http">http</option>
                <option value="https">https</option>
              </select>
            </label>
            <label>
              Port
              <input v-model.number="newServerPort" type="number" min="1" max="65535" />
            </label>
            <label>
              Notes
              <input v-model="newServerNotes" placeholder="Optional operator note" />
            </label>
          </div>

          <div class="actions">
            <button :disabled="!canAddServer" @click="addServerProfile">Add server profile</button>
          </div>
        </section>

        <section class="surface-card">
          <div class="card-heading">
            <div>
              <h2>Connection</h2>
              <p class="muted">Connect to the selected WinPassage service. Admin tokens stay session-only and are not stored with the server profile.</p>
            </div>
            <span class="status-pill" :class="adminToken ? 'success' : 'warning'">{{ adminToken ? 'Token provided' : 'Token required' }}</span>
          </div>
          <div class="grid two">
            <label>
              Active server URL
              <input v-model="serverUrl" placeholder="http://192.168.1.10:4487" />
            </label>
            <label>
              Admin token
              <input v-model="adminToken" type="password" autocomplete="off" placeholder="Session-only operator token" />
            </label>
          </div>
          <div class="actions">
            <button :disabled="loading || !adminToken" @click="refreshAll">Load users & sessions</button>
            <button class="secondary" :disabled="loading" @click="persistSettings">Save active server address</button>
          </div>
        </section>

        <p v-if="message" class="notice success">{{ message }}</p>
        <p v-if="error" class="notice error">{{ error }}</p>

        <section class="panel-grid">
          <article class="surface-card">
            <div class="card-heading">
              <div>
                <h2 id="admin-users">Local users</h2>
                <p class="muted">The list is loaded from the selected server's local Windows accounts, not from a WinPassage database.</p>
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
                <p>Select a server profile, enter the admin token, then load local users from that machine.</p>
              </div>
            </div>
          </article>

          <article class="surface-card">
            <div class="card-heading">
              <div>
                <h2>Create local account</h2>
                <p class="muted">Creates a real Windows local user on the active central machine.</p>
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
              <button :disabled="loading || !canCreate" @click="createUser">Create user on active server</button>
            </div>
          </article>
        </section>

        <section class="panel-grid">
          <article class="danger-card">
            <div class="card-heading">
              <div>
                <h2>Account control</h2>
                <p class="muted">Change status, reset password, and manage administrator access for the selected user on the active server.</p>
              </div>
              <span class="status-pill danger">Privileged</span>
            </div>

            <p class="notice warning">Every operation below changes the Windows account directly on {{ activeServerProfile?.name || serverUrl }} and should include an audit reason.</p>

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
                <p class="muted">Requires typing the exact username before the account is removed from the active Windows server.</p>
              </div>
              <span class="status-pill danger">Destructive</span>
            </div>

            <p class="notice error">Deleting a user removes the local Windows account from {{ activeServerProfile?.name || serverUrl }}. Profile deletion is intentionally separated and must be enabled only after profile API handling is production-ready.</p>
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
              <p class="muted">Use logoff for stale or offboarding sessions on the active server before deleting or disabling an account.</p>
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
              <p>Load sessions from the active server to manage active sign-ins.</p>
            </div>
          </div>
        </section>
      </section>
    </section>
  </main>
</template>
