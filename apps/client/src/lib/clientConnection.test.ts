import { describe, expect, it } from 'vitest';
import {
  canUnlockAdvancedServerSettings,
  normalizeServerUrl,
  validateServerUrl,
} from './clientConnection';

describe('client connection settings', () => {
  it('keeps server changes behind the explicit confirmation phrase', () => {
    expect(canUnlockAdvancedServerSettings('CHANGE SERVER')).toBe(true);
    expect(canUnlockAdvancedServerSettings('change server')).toBe(false);
    expect(canUnlockAdvancedServerSettings('')).toBe(false);
  });

  it('normalizes trailing slashes without hiding the target server', () => {
    expect(normalizeServerUrl(' https://192.168.1.10:4487/// ')).toBe('https://192.168.1.10:4487');
  });

  it('accepts a normal server URL', () => {
    expect(validateServerUrl('https://office-server.local:4487')).toEqual([]);
  });

  it('rejects missing protocol and URL fragments', () => {
    expect(validateServerUrl('office-server.local:4487#secret')).toEqual([
      'Server URL must start with http:// or https:// and include a host.',
      'Server URL must not include query strings or fragments.',
    ]);
  });
});
