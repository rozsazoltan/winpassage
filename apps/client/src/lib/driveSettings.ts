import type { DriveMapping } from '../types';

export interface StoredDriveMapping extends DriveMapping {
  id: string;
}

export function normalizeDriveLetter(value: string): string {
  const letter = value.trim().replace(/[:\\/]/g, '').toUpperCase();
  return letter ? `${letter}:` : '';
}

export function loadStoredDrives(raw: string | null): StoredDriveMapping[] {
  if (!raw) return [];

  try {
    const parsed = JSON.parse(raw) as StoredDriveMapping[];
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((drive) => drive.letter?.trim() && drive.remote_path?.trim());
  } catch {
    return [];
  }
}
