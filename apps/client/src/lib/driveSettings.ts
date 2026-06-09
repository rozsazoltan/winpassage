import type { DriveMapping } from '../types';

export interface StoredDriveMapping extends DriveMapping {
  id: string;
}

type StoredDriveCandidate = {
  id?: unknown;
  letter?: unknown;
  remote_path?: unknown;
};

function isRecord(value: unknown): value is StoredDriveCandidate {
  return typeof value === 'object' && value !== null;
}

export function normalizeDriveLetter(value: string): string {
  const letter = value.trim().replace(/[:\\/]/g, '').toUpperCase();
  return letter ? `${letter}:` : '';
}

export function loadStoredDrives(raw: string | null): StoredDriveMapping[] {
  if (!raw) return [];

  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];

    return parsed.flatMap((drive) => {
      if (!isRecord(drive)) return [];
      if (typeof drive.id !== 'string') return [];
      if (typeof drive.letter !== 'string') return [];
      if (typeof drive.remote_path !== 'string') return [];

      const id = drive.id.trim();
      const letter = normalizeDriveLetter(drive.letter);
      const remotePath = drive.remote_path.trim();

      if (!id || !letter || !remotePath) return [];

      return [{ id, letter, remote_path: remotePath }];
    });
  } catch {
    return [];
  }
}
