import { describe, expect, it } from 'vitest';
import type { ServerProfile } from '../types';
import { buildServerBaseUrl, normalizeServerHost, validateServerProfile } from './serverProfiles';

const validProfile: ServerProfile = {
  id: 'office-main',
  name: 'Office Main',
  network_name: 'Budapest office',
  protocol: 'https',
  host: '192.168.1.10',
  port: 4487,
  notes: null,
};

describe('server profile helpers', () => {
  it('normalizes hosts copied with a protocol prefix', () => {
    expect(normalizeServerHost(' https://server.local/ ')).toBe('server.local');
  });

  it('builds the base URL from a server profile', () => {
    expect(buildServerBaseUrl(validProfile)).toBe('https://192.168.1.10:4487');
  });

  it('accepts a valid IP based profile', () => {
    expect(validateServerProfile(validProfile)).toEqual([]);
  });

  it('rejects paths and invalid ports because profiles must point to a server root', () => {
    expect(
      validateServerProfile({
        ...validProfile,
        host: 'server.local/api',
        port: 70000,
      }),
    ).toEqual([
      'Server host must be an IP address or DNS name without a path.',
      'Server port must be between 1 and 65535.',
    ]);
  });
});
