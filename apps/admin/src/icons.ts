// Inline SVG icon helper inspired by Nutrino's local Lucide icon pack approach.
// Nodes are small, dependency-free subsets of Lucide-style SVG data.

type IconAttrs = Record<string, string>;
type IconNode = ReadonlyArray<readonly [string, IconAttrs]>;

const iconNodes = {
  overview: [["path", { d: "M3 10.5 12 3l9 7.5" }], ["path", { d: "M5 10v10h14V10" }], ["path", { d: "M9 20v-6h6v6" }]],
  password: [["rect", { x: "4", y: "11", width: "16", height: "10", rx: "2" }], ["path", { d: "M8 11V8a4 4 0 0 1 8 0v3" }], ["path", { d: "M12 15v2" }]],
  drives: [["path", { d: "M6 3h12l3 9v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-7Z" }], ["path", { d: "M3 12h18" }], ["circle", { cx: "8", cy: "17", r: "1" }], ["circle", { cx: "12", cy: "17", r: "1" }]],
  settings: [["path", { d: "M9.7 4.1a2.3 2.3 0 0 1 4.6 0 2.3 2.3 0 0 0 3.3 1.9 2.3 2.3 0 0 1 2.3 4 2.3 2.3 0 0 0 0 4 2.3 2.3 0 0 1-2.3 4 2.3 2.3 0 0 0-3.3 1.9 2.3 2.3 0 0 1-4.6 0 2.3 2.3 0 0 0-3.3-1.9 2.3 2.3 0 0 1-2.3-4 2.3 2.3 0 0 0 0-4 2.3 2.3 0 0 1 2.3-4 2.3 2.3 0 0 0 3.3-1.9" }], ["circle", { cx: "12", cy: "12", r: "3" }]],
  users: [["path", { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }], ["circle", { cx: "9", cy: "7", r: "4" }], ["path", { d: "M22 21v-2a4 4 0 0 0-3-3.87" }], ["path", { d: "M16 3.13a4 4 0 0 1 0 7.75" }]],
  sessions: [["rect", { x: "3", y: "4", width: "18", height: "12", rx: "2" }], ["path", { d: "M8 20h8" }], ["path", { d: "M12 16v4" }]],
  servers: [["rect", { x: "3", y: "3", width: "18", height: "7", rx: "2" }], ["rect", { x: "3", y: "14", width: "18", height: "7", rx: "2" }], ["path", { d: "M7 7h.01" }], ["path", { d: "M7 18h.01" }]],
  refresh: [["path", { d: "M3 12a9 9 0 0 1 15.3-6.4L21 8" }], ["path", { d: "M21 3v5h-5" }], ["path", { d: "M21 12a9 9 0 0 1-15.3 6.4L3 16" }], ["path", { d: "M3 21v-5h5" }]],
  plus: [["path", { d: "M5 12h14" }], ["path", { d: "M12 5v14" }]],
  play: [["path", { d: "m6 3 14 9-14 9Z" }]],
  stop: [["rect", { x: "6", y: "6", width: "12", height: "12", rx: "2" }]],
  edit: [["path", { d: "M21.2 6.8a1 1 0 0 0-4-4L4 16l-2 6 6-2Z" }], ["path", { d: "m15 5 4 4" }]],
  trash: [["path", { d: "M3 6h18" }], ["path", { d: "M8 6V4h8v2" }], ["path", { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" }], ["path", { d: "M10 11v6" }], ["path", { d: "M14 11v6" }]],
  shield: [["path", { d: "M20 13c0 5-3.5 7.5-7.7 9a1 1 0 0 1-.6 0C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.2-2.7a1.2 1.2 0 0 1 1.6 0C14.5 3.8 17 5 19 5a1 1 0 0 1 1 1Z" }]],
  info: [["circle", { cx: "12", cy: "12", r: "10" }], ["path", { d: "M12 16v-4" }], ["path", { d: "M12 8h.01" }]],
  check: [["path", { d: "m20 6-11 11-5-5" }]],
  warning: [["path", { d: "m21.7 18-8.7-15a1.2 1.2 0 0 0-2 0L2.3 18a1.2 1.2 0 0 0 1 1.8h17.4a1.2 1.2 0 0 0 1-1.8Z" }], ["path", { d: "M12 9v4" }], ["path", { d: "M12 17h.01" }]],
  key: [["circle", { cx: "7.5", cy: "15.5", r: "5.5" }], ["path", { d: "m12 11 8-8" }], ["path", { d: "m17 3 4 4" }], ["path", { d: "m15 5 4 4" }]],
  network: [["rect", { x: "16", y: "16", width: "6", height: "6", rx: "1" }], ["rect", { x: "2", y: "16", width: "6", height: "6", rx: "1" }], ["rect", { x: "9", y: "2", width: "6", height: "6", rx: "1" }], ["path", { d: "M12 8v4" }], ["path", { d: "M5 16v-2a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v2" }]],
  chevronRight: [["path", { d: "m9 18 6-6-6-6" }]],
  x: [["path", { d: "M18 6 6 18" }], ["path", { d: "m6 6 12 12" }]],
  sun: [["circle", { cx: "12", cy: "12", r: "4" }], ["path", { d: "M12 2v2" }], ["path", { d: "M12 20v2" }], ["path", { d: "m4.9 4.9 1.4 1.4" }], ["path", { d: "m17.7 17.7 1.4 1.4" }], ["path", { d: "M2 12h2" }], ["path", { d: "M20 12h2" }], ["path", { d: "m6.3 17.7-1.4 1.4" }], ["path", { d: "m19.1 4.9-1.4 1.4" }]],
  moon: [["path", { d: "M12 3a6 6 0 0 0 9 7.4A9 9 0 1 1 12 3Z" }]],
  monitor: [["rect", { x: "3", y: "4", width: "18", height: "12", rx: "2" }], ["path", { d: "M8 20h8" }], ["path", { d: "M12 16v4" }]],
  logOut: [["path", { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }], ["path", { d: "m16 17 5-5-5-5" }], ["path", { d: "M21 12H9" }]],
} as const satisfies Record<string, IconNode>;

export type IconName = keyof typeof iconNodes;

function escapeAttribute(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/"/g, '&quot;');
}

export function lucideSvg(name: IconName, options: { strokeWidth?: number; className?: string } = {}): string {
  const node = iconNodes[name] ?? iconNodes.info;
  const strokeWidth = String(options.strokeWidth ?? 2);
  const className = ['lucide-svg', options.className ?? ''].filter(Boolean).join(' ');
  const children = node
    .map(([tag, attrs]) => {
      const serialized = Object.entries(attrs)
        .map(([key, value]) => `${key}="${escapeAttribute(value)}"`)
        .join(' ');
      return `<${tag} ${serialized}/>`;
    })
    .join('');

  return `<svg class="${className}" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="${strokeWidth}" stroke-linecap="round" stroke-linejoin="round">${children}</svg>`;
}
