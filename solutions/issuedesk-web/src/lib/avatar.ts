/**
 * Deterministic "beam"-style avatars (à la boring-avatars) — a modern take on
 * the old GitHub identicon. Same seed → same colorful abstract face, with zero
 * dependencies. Seed by a stable string (user id, falling back to username).
 */
const SIZE = 36;

// Vibrant palette aligned with the app's indigo/sky/emerald/amber/rose theme.
const PALETTE = ['#6366f1', '#0ea5e9', '#10b981', '#f59e0b', '#f43f5e'];

function hashCode(name: string): number {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    const ch = name.charCodeAt(i);
    hash = (hash << 5) - hash + ch;
    hash |= 0;
  }
  return Math.abs(hash);
}

function getDigit(n: number, ntn: number): number {
  return Math.floor((n / Math.pow(10, ntn)) % 10);
}
function getBoolean(n: number, ntn: number): boolean {
  return getDigit(n, ntn) % 2 === 0;
}
function getUnit(n: number, range: number, index?: number): number {
  const value = n % range;
  if (index !== undefined && getDigit(n, index) % 2 === 0) return -value;
  return value;
}
function getContrast(hex: string): string {
  const h = hex.replace('#', '');
  const r = parseInt(h.substring(0, 2), 16);
  const g = parseInt(h.substring(2, 4), 16);
  const b = parseInt(h.substring(4, 6), 16);
  const yiq = (r * 299 + g * 587 + b * 114) / 1000;
  return yiq >= 128 ? '#000000' : '#ffffff';
}

export interface BeamData {
  wrapperColor: string;
  faceColor: string;
  backgroundColor: string;
  wrapperTranslateX: number;
  wrapperTranslateY: number;
  wrapperRotate: number;
  wrapperScale: number;
  isMouthOpen: boolean;
  isCircle: boolean;
  eyeSpread: number;
  mouthSpread: number;
  faceRotate: number;
  faceTranslateX: number;
  faceTranslateY: number;
}

export function beamData(seed: string): BeamData {
  const n = hashCode(seed || 'anon');
  const range = PALETTE.length;
  const wrapperColor = PALETTE[n % range];
  const preTranslateX = getUnit(n, 10, 1);
  const wrapperTranslateX = preTranslateX < 5 ? preTranslateX + SIZE / 9 : preTranslateX;
  const preTranslateY = getUnit(n, 10, 2);
  const wrapperTranslateY = preTranslateY < 5 ? preTranslateY + SIZE / 9 : preTranslateY;

  return {
    wrapperColor,
    faceColor: getContrast(wrapperColor),
    backgroundColor: PALETTE[(n + 13) % range],
    wrapperTranslateX,
    wrapperTranslateY,
    wrapperRotate: getUnit(n, 360),
    wrapperScale: 1 + getUnit(n, SIZE / 12) / 10,
    isMouthOpen: getBoolean(n, 2),
    isCircle: getBoolean(n, 1),
    eyeSpread: getUnit(n, 5),
    mouthSpread: getUnit(n, 3),
    faceRotate: getUnit(n, 10, 3),
    faceTranslateX: wrapperTranslateX > SIZE / 6 ? wrapperTranslateX / 2 : getUnit(n, 8, 1),
    faceTranslateY: wrapperTranslateY > SIZE / 6 ? wrapperTranslateY / 2 : getUnit(n, 7, 2)
  };
}

let uid = 0;
export function nextMaskId(): string {
  return `beam-mask-${++uid}`;
}
