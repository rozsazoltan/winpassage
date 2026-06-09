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
