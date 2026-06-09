export interface PasswordChangeResponse {
  success: boolean;
  message: string;
  request_id: string;
}

export interface ClientServerSettings {
  server_url: string;
  locked: boolean;
}

export interface DriveMapping {
  letter: string;
  remote_path: string;
}

export interface DriveReconnectResult {
  letter: string;
  remote_path: string;
  success: boolean;
  message: string;
}

export interface ReconnectMappedDrivesResponse {
  results: DriveReconnectResult[];
}

export interface DriveOperationResponse {
  success: boolean;
  letter: string;
  remote_path?: string;
  message: string;
}
