/** Fixed-height history geometry. Graph layout always uses the complete loaded history. */
export const HISTORY_ROW_HEIGHT = 48;
export const HISTORY_OVERSCAN = 8;
export interface HistoryWindow {
  start: number;
  end: number;
  totalHeight: number;
}
export function historyWindow(
  count: number,
  scrollTop: number,
  height: number,
): HistoryWindow {
  count = Math.max(0, Math.floor(Number.isFinite(count) ? count : 0));
  const top = Math.max(0, Number.isFinite(scrollTop) ? scrollTop : 0);
  const viewport = Math.max(1, Number.isFinite(height) ? height : 1);
  const first = Math.min(
    Math.max(0, count - 1),
    Math.floor(top / HISTORY_ROW_HEIGHT),
  );
  const last = Math.min(
    count,
    Math.ceil((top + viewport) / HISTORY_ROW_HEIGHT),
  );
  return {
    start: Math.max(0, first - HISTORY_OVERSCAN),
    end: Math.min(count, Math.max(first + 1, last) + HISTORY_OVERSCAN),
    totalHeight: count * HISTORY_ROW_HEIGHT,
  };
}
export function historyScrollTarget(
  index: number,
  scrollTop: number,
  height: number,
): number {
  const top = index * HISTORY_ROW_HEIGHT,
    bottom = top + HISTORY_ROW_HEIGHT;
  if (top < scrollTop) return top;
  if (bottom > scrollTop + height) return Math.max(0, bottom - height);
  return scrollTop;
}
/** Keep a focused row alive while manually scrolling; pending keyboard focus may add one more. */
export function historyIndices(
  window: HistoryWindow,
  count: number,
  retained: readonly (number | null)[],
): number[] {
  const indices = new Set<number>();
  for (let i = Math.max(0, window.start); i < Math.min(count, window.end); i++)
    indices.add(i);
  for (const i of retained)
    if (i !== null && Number.isInteger(i) && i >= 0 && i < count)
      indices.add(i);
  return [...indices].sort((a, b) => a - b);
}
export function historyNavigationIndex(
  key: string,
  current: number,
  count: number,
  height: number,
): number | null {
  if (!count) return null;
  const page = Math.max(1, Math.floor(height / HISTORY_ROW_HEIGHT));
  const next =
    key === "ArrowDown"
      ? current + 1
      : key === "ArrowUp"
        ? current - 1
        : key === "PageDown"
          ? current + page
          : key === "PageUp"
            ? current - page
            : key === "Home"
              ? 0
              : key === "End"
                ? count - 1
                : null;
  return next === null ? null : Math.max(0, Math.min(count - 1, next));
}
