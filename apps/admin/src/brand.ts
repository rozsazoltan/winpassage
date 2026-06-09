export function winPassageMarkSvg(className = 'brand-svg'): string {
  return `
    <svg class="${className}" viewBox="0 0 96 96" role="img" aria-label="WinPassage secure bridge mark" fill="none" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <linearGradient id="wp-bridge-metal" x1="22" y1="18" x2="72" y2="82" gradientUnits="userSpaceOnUse">
          <stop stop-color="#F8FBFF" />
          <stop offset="0.46" stop-color="#BAC8D9" />
          <stop offset="1" stop-color="#61738B" />
        </linearGradient>
        <linearGradient id="wp-bridge-blue" x1="48" y1="63" x2="48" y2="91" gradientUnits="userSpaceOnUse">
          <stop stop-color="#4D90E8" stop-opacity="0.9" />
          <stop offset="1" stop-color="#0B1B32" stop-opacity="0" />
        </linearGradient>
        <filter id="wp-soft-shadow" x="4" y="8" width="88" height="84" filterUnits="userSpaceOnUse" color-interpolation-filters="sRGB">
          <feDropShadow dx="0" dy="3" stdDeviation="2.2" flood-color="#06101F" flood-opacity="0.25" />
        </filter>
      </defs>
      <g filter="url(#wp-soft-shadow)">
        <path d="M18 63C29 59 35 50 40 42" stroke="url(#wp-bridge-metal)" stroke-width="6" stroke-linecap="round" />
        <path d="M78 63C67 59 61 50 56 42" stroke="url(#wp-bridge-metal)" stroke-width="6" stroke-linecap="round" />
        <path d="M18 67H78" stroke="url(#wp-bridge-metal)" stroke-width="5" stroke-linecap="round" />
        <path d="M26 62V68M34 53V68M62 53V68M70 62V68" stroke="#DCE8F5" stroke-width="2.8" stroke-linecap="round" opacity="0.82" />
        <path d="M35 46V35C35 24 41 17 48 17C55 17 61 24 61 35V46" stroke="url(#wp-bridge-metal)" stroke-width="8" stroke-linecap="round" />
        <path d="M33 44H63C66 44 68 46 68 49V78H28V49C28 46 30 44 33 44Z" fill="url(#wp-bridge-metal)" />
        <path d="M40 78V68C40 59 45 54 48 54C51 54 56 59 56 68V78" fill="#071426" opacity="0.92" />
        <path d="M48 52m-5 0a5 5 0 1 0 10 0a5 5 0 1 0-10 0M48 56L44 66H52L48 56Z" fill="#071426" />
        <path d="M33 84C41 77 55 77 63 84L64 90H32L33 84Z" fill="url(#wp-bridge-blue)" />
      </g>
    </svg>`;
}

export function winPassageInlineSvg(className = 'inline-brand-svg'): string {
  return winPassageMarkSvg(className);
}
