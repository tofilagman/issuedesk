// Board card aging: how long a ticket has sat in its current status, so stale
// work is easy to spot. Done tickets are complete and never age.

export type AgingLevel = 'fresh' | 'warn' | 'stale';

export interface Aging {
  /** Short human label, e.g. "5h", "3d", "2w". */
  label: string;
  level: AgingLevel;
  /** Tooltip text. */
  title: string;
}

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

// Days in the current status before a card is flagged. Done (status 4) never ages.
const WARN_DAYS = 3;
const STALE_DAYS = 7;
const DONE_STATUS = 4;

function humanize(ms: number): string {
  const mins = Math.floor(ms / MINUTE);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m`;
  const hrs = Math.floor(ms / HOUR);
  if (hrs < 24) return `${hrs}h`;
  const days = Math.floor(ms / DAY);
  if (days < 14) return `${days}d`;
  return `${Math.floor(days / 7)}w`;
}

/**
 * Aging for a board card, or `null` when there is nothing to show (Done, or a
 * fresh ticket below the warn threshold).
 */
export function ticketAging(
  statusSince: string,
  status: number,
  statusLabel: string,
  now: number
): Aging | null {
  if (status === DONE_STATUS) return null;
  const since = new Date(statusSince).getTime();
  if (!Number.isFinite(since)) return null;

  const ms = Math.max(0, now - since);
  const days = ms / DAY;
  if (days < WARN_DAYS) return null; // fresh — keep the board uncluttered

  const level: AgingLevel = days >= STALE_DAYS ? 'stale' : 'warn';
  return { label: humanize(ms), level, title: `In ${statusLabel} for ${humanize(ms)}` };
}
