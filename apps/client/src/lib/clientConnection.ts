const SERVER_URL_PATTERN = /^https?:\/\/[^\s/$.?#].[^\s]*$/i;

export function normalizeServerUrl(value: string): string {
  return value.trim().replace(/\/+$/, '');
}

export function canUnlockAdvancedServerSettings(confirmation: string): boolean {
  return confirmation.trim() === 'CHANGE SERVER';
}

export function validateServerUrl(value: string): string[] {
  const normalized = normalizeServerUrl(value);
  const errors: string[] = [];

  if (!normalized) {
    errors.push('Server URL is required.');
  }

  if (normalized && !SERVER_URL_PATTERN.test(normalized)) {
    errors.push('Server URL must start with http:// or https:// and include a host.');
  }

  if (normalized.includes('?') || normalized.includes('#')) {
    errors.push('Server URL must not include query strings or fragments.');
  }

  return errors;
}
