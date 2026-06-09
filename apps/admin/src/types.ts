export interface ServerProfile {
  id: string;
  name: string;
  network_name: string;
  protocol: 'http' | 'https';
  host: string;
  port: number;
  notes: string | null;
}

export interface LocalUserSummary {
  username: string;
  full_name: string | null;
  disabled: boolean;
  password_required: boolean;
  is_administrator: boolean;
  active_session_count: number;
}

export interface ListUsersResponse {
  users: LocalUserSummary[];
}

export interface LocalSessionSummary {
  session_id: number;
  username: string | null;
  domain: string | null;
  state: string;
  client_name: string | null;
  is_console: boolean;
}

export interface ListSessionsResponse {
  sessions: LocalSessionSummary[];
}

export interface ActionResponse {
  success: boolean;
  message: string;
  request_id: string;
}

export type PasswordChangeResponse = ActionResponse;

export interface AdminHostStatus {
  is_windows: boolean;
  is_admin_account: boolean;
  is_elevated: boolean;
  service_installed: boolean;
  install_dir: string;
  executable_dir: string | null;
  message: string;
}

export interface ServerInstallResult {
  success: boolean;
  message: string;
  install_dir: string;
  server_url: string;
}
