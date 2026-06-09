import type { ServerProfile } from '../types';

const HOST_PATTERN = /^[a-zA-Z0-9.-]+$/;

export function normalizeServerHost(host: string): string {
  return host.trim().replace(/^https?:\/\//i, '').replace(/\/+$/, '');
}

export function buildServerBaseUrl(profile: ServerProfile): string {
  const protocol = profile.protocol === 'https' ? 'https' : 'http';
  const host = normalizeServerHost(profile.host);

  return `${protocol}://${host}:${profile.port}`;
}

export function validateServerProfile(profile: ServerProfile): string[] {
  const errors: string[] = [];
  const host = normalizeServerHost(profile.host);

  if (!profile.name.trim()) {
    errors.push('Server profile name is required.');
  }

  if (!profile.network_name.trim()) {
    errors.push('Network name is required.');
  }

  if (!host) {
    errors.push('Server host or IP address is required.');
  }

  if (host.includes('/') || !HOST_PATTERN.test(host)) {
    errors.push('Server host must be an IP address or DNS name without a path.');
  }

  if (!Number.isInteger(profile.port) || profile.port < 1 || profile.port > 65535) {
    errors.push('Server port must be between 1 and 65535.');
  }

  if (profile.protocol !== 'http' && profile.protocol !== 'https') {
    errors.push('Server protocol must be http or https.');
  }

  return errors;
}
