import { useCallback, useLayoutEffect, useRef, useState } from "react";
import { historyWindow } from "./historyWindow";
export function useHistoryWindow(count: number) {
  const scroll = useRef<HTMLDivElement>(null);
  const [window, setWindow] = useState(() => historyWindow(count, 0, 600));
  const measure = useCallback(() => {
    const host = scroll.current;
    if (!host) return;
    const next = historyWindow(count, host.scrollTop, host.clientHeight);
    setWindow((previous) =>
      previous.start === next.start &&
      previous.end === next.end &&
      previous.totalHeight === next.totalHeight
        ? previous
        : next,
    );
  }, [count]);
  useLayoutEffect(() => {
    measure();
    const host = scroll.current;
    if (!host) return;
    const observer = new ResizeObserver(measure);
    observer.observe(host);
    return () => observer.disconnect();
  }, [measure]);
  return { scroll, window, measure };
}
