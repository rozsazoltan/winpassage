import { describe, expect, it } from 'vitest';
import { loadStoredDrives, normalizeDriveLetter } from './driveSettings';

describe('client drive settings', () => {
  it('starts with no default drives when storage is empty', () => {
    expect(loadStoredDrives(null)).toEqual([]);
  });

  it('normalizes drive letters', () => {
    expect(normalizeDriveLetter('s')).toBe('S:');
    expect(normalizeDriveLetter('g:')).toBe('G:');
    expect(normalizeDriveLetter('i:/')).toBe('I:');
  });

  it('drops invalid stored drive entries', () => {
    const stored = JSON.stringify([
      { id: '1', letter: 'S:', remote_path: '\\\\server\\share' },
      { id: '2', letter: '', remote_path: 'x' },
      { id: '3', letter: 'T:', remote_path: '' },
      { id: '4', letter: 'R:', remote_path: 42 },
      { id: '', letter: 'U:', remote_path: '\\\\server\\empty-id' },
      null,
    ]);

    expect(loadStoredDrives(stored)).toEqual([
      { id: '1', letter: 'S:', remote_path: '\\\\server\\share' },
    ]);
  });

  it('returns no drives for invalid JSON', () => {
    expect(loadStoredDrives('[{"remote_path":"\\server\share"}]')).toEqual([]);
  });
});
