export function winPassageMarkSvg(className = 'brand-svg'): string {
  return `
    <svg class="${className}" viewBox="0 0 96 96" role="img" aria-label="WinPassage bridge lock mark">
      <defs>
        <linearGradient id="wp-bg" x1="15" y1="10" x2="82" y2="90" gradientUnits="userSpaceOnUse">
          <stop stop-color="#1E3A5F" />
          <stop offset="1" stop-color="#071426" />
        </linearGradient>
        <linearGradient id="wp-metal" x1="29" y1="18" x2="70" y2="82" gradientUnits="userSpaceOnUse">
          <stop stop-color="#F4F8FC" />
          <stop offset="0.48" stop-color="#AFC1D6" />
          <stop offset="1" stop-color="#5F7590" />
        </linearGradient>
        <linearGradient id="wp-blue" x1="48" y1="64" x2="48" y2="92" gradientUnits="userSpaceOnUse">
          <stop stop-color="#60A5FA" stop-opacity="0.85" />
          <stop offset="1" stop-color="#0B1730" stop-opacity="0.15" />
        </linearGradient>
      </defs>
      <rect x="4" y="4" width="88" height="88" rx="22" fill="url(#wp-bg)" />
      <path d="M28 56 C38 49 58 49 68 56" stroke="#D8E3EF" stroke-width="3.5" stroke-linecap="round" opacity="0.82" />
      <path d="M17 61 C30 55 35 43 44 43" stroke="url(#wp-metal)" stroke-width="5" stroke-linecap="round" fill="none" />
      <path d="M79 61 C66 55 61 43 52 43" stroke="url(#wp-metal)" stroke-width="5" stroke-linecap="round" fill="none" />
      <path d="M20 65 H76" stroke="#6F86A1" stroke-width="4" stroke-linecap="round" opacity="0.75" />
      <path d="M30 57 V66 M38 49 V66 M58 49 V66 M66 57 V66" stroke="#D3E0EF" stroke-width="2.6" stroke-linecap="round" opacity="0.8" />
      <path d="M35 45 V36 C35 26 41 19 48 19 C55 19 61 26 61 36 V45" stroke="url(#wp-metal)" stroke-width="8" stroke-linecap="round" fill="none" />
      <path d="M34 43 H62 C65 43 67 45 67 48 V76 H29 V48 C29 45 31 43 34 43Z" fill="url(#wp-metal)" />
      <path d="M41 77 V67 C41 59 45 55 48 55 C51 55 55 59 55 67 V77" fill="#071426" opacity="0.9" />
      <path d="M48 51 m-5 0a5 5 0 1 0 10 0a5 5 0 1 0-10 0 M48 55 L44 65 H52Z" fill="#071426" />
      <path d="M33 83 C41 76 55 76 63 83 L63 90 H33Z" fill="url(#wp-blue)" opacity="0.7" />
      <rect x="4.75" y="4.75" width="86.5" height="86.5" rx="21" fill="none" stroke="white" stroke-opacity="0.12" />
    </svg>`;
}

export function winPassageInlineSvg(className = 'inline-brand-svg'): string {
  return winPassageMarkSvg(className);
}
