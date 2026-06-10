export function winPassageMarkSvg(className = 'brand-svg'): string {
  return `
    <svg class="${className}" viewBox="0 0 96 96" role="img" aria-label="WinPassage secure bridge mark" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M29 48V38C29 26 37 18 48 18C59 18 67 26 67 38V48" stroke="currentColor" stroke-width="8" stroke-linecap="round" opacity="0.96" />
      <path d="M26 47H70C74 47 77 50 77 54V77C77 80 75 82 72 82H24C21 82 19 80 19 77V54C19 50 22 47 26 47Z" fill="currentColor" opacity="0.94" />
      <path d="M28 70C38 60 58 60 68 70" stroke="var(--surface, #fff)" stroke-width="6" stroke-linecap="round" opacity="0.92" />
      <path d="M18 68H78" stroke="var(--surface, #fff)" stroke-width="5" stroke-linecap="round" opacity="0.86" />
      <path d="M31 63V72M39 59V72M57 59V72M65 63V72" stroke="var(--surface, #fff)" stroke-width="3" stroke-linecap="round" opacity="0.7" />
      <path d="M48 55m-5 0a5 5 0 1 0 10 0a5 5 0 1 0-10 0" fill="var(--surface, #fff)" />
      <path d="M48 59L43 72H53L48 59Z" fill="var(--surface, #fff)" />
    </svg>`;
}

export function winPassageInlineSvg(className = 'inline-brand-svg'): string {
  return winPassageMarkSvg(className);
}
