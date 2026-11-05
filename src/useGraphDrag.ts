import { useEffect, useRef, useState, type PointerEvent } from "react";
import type { GraphSource } from "./graphInteractions";
/** Pointer capture keeps graph gestures independent of WebView native file dragging. */
export function useGraphDrag(
  context: string,
  enabled: boolean,
  finish: (source: GraphSource, target: string, context: string) => void,
) {
  type Session = {
    source: GraphSource;
    context: string;
    x: number;
    y: number;
    active: boolean;
  };
  const session = useRef<Session | null>(null);
  const suppressClick = useRef(false);
  const [drag, setDrag] = useState<Session | null>(null);
  const [dropTarget, setDropTarget] = useState<string | null>(null);
  function endDrag() {
    session.current = null;
    setDrag(null);
    setDropTarget(null);
  }
  useEffect(() => {
    endDrag();
  }, [context, enabled]);
  useEffect(() => {
    const cancel = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") endDrag();
    };
    window.addEventListener("keydown", cancel);
    window.addEventListener("blur", endDrag);
    return () => {
      window.removeEventListener("keydown", cancel);
      window.removeEventListener("blur", endDrag);
    };
  }, []);
  function target(event: PointerEvent) {
    return (
      document
        .elementFromPoint(event.clientX, event.clientY)
        ?.closest<HTMLElement>("[data-graph-target]")?.dataset.graphTarget ??
      null
    );
  }
  function sourceEvents(source: GraphSource) {
    return {
      draggable: false,
      onPointerDown(event: PointerEvent<HTMLElement>) {
        if (!enabled || event.button !== 0 || !event.isPrimary) return;
        suppressClick.current = false;
        session.current = {
          source,
          context,
          x: event.clientX,
          y: event.clientY,
          active: false,
        };
        event.currentTarget.setPointerCapture(event.pointerId);
      },
      onPointerMove(event: PointerEvent<HTMLElement>) {
        const current = session.current;
        if (!current) return;
        if (
          !current.active &&
          Math.hypot(event.clientX - current.x, event.clientY - current.y) < 6
        )
          return;
        if (!current.active) {
          current.active = true;
          setDrag({ ...current });
        }
        suppressClick.current = true;
        setDropTarget(target(event));
      },
      onPointerUp(event: PointerEvent<HTMLElement>) {
        const current = session.current,
          destination = target(event);
        endDrag();
        if (
          current?.active &&
          destination &&
          current.context === context &&
          enabled
        )
          finish(current.source, destination, current.context);
      },
      onPointerCancel: endDrag,
      onLostPointerCapture: endDrag,
    };
  }
  return { drag, dropTarget, endDrag, sourceEvents, suppressClick };
}

export type GraphGesture = ReturnType<typeof useGraphDrag>;
