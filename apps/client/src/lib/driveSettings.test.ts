import { describe, expect, it } from 'vitest';
import { loadStoredDrives, normalizeDriveLetter } from './driveSettings';

describe('client drive settings', () => {
  it('starts with no default drives when storage is empty', () => {
    expect(loadStoredDrives(null)).toEqual([]);
  });

  it('normalizes drive letters', () => {
    expect(normalizeDriveLetter('s')).toBe('S:');
    expect(normalizeDriveLetter('g:')).toBe('G:');
  });

  it('drops invalid stored drive entries', () => {
    expect(loadStoredDrives('[{"id":"1","letter":"S:","remote_path":"\\\\server\\share"},{"id":"2","letter":"","remote_path":"x"}]')).toEqual([
      { id: '1', letter: 'S:', remote_path: '\\\\server\\share' },
    ]);
  });
});
