export interface LocalUserSummary {
  username: string;
  full_name: string | null;
  disabled: boolean;
  password_required: boolean;
}

export interface ListUsersResponse {
  users: LocalUserSummary[];
}

export interface PasswordChangeResponse {
  success: boolean;
  message: string;
  request_id: string;
}
