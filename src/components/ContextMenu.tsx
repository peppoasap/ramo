import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
export interface MenuItem {
  label: string;
  run: () => void;
  disabled?: boolean;
  danger?: boolean;
}
export interface MenuState {
  x: number;
  y: number;
  title: string;
  items: MenuItem[];
}
/** One menu for pointer and keyboard invocation; focus returns to its trigger. */
export function ContextMenu({
  menu,
  close,
}: {
  menu: MenuState;
  close: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const trigger = useRef(document.activeElement);
  const [position, setPosition] = useState({ left: menu.x, top: menu.y });
  useLayoutEffect(() => {
    const bounds = ref.current!.getBoundingClientRect();
    setPosition({
      left: Math.max(8, Math.min(menu.x, innerWidth - bounds.width - 8)),
      top: Math.max(8, Math.min(menu.y, innerHeight - bounds.height - 8)),
    });
    ref.current
      ?.querySelector<HTMLButtonElement>("button:not(:disabled)")
      ?.focus();
  }, [menu]);
  useEffect(() => {
    const previous = trigger.current;
    const element = ref.current;
    function outside(event: PointerEvent) {
      if (!ref.current?.contains(event.target as Node)) close();
    }
    function scroll() {
      close();
    }
    document.addEventListener("pointerdown", outside, true);
    window.addEventListener("resize", scroll);
    return () => {
      document.removeEventListener("pointerdown", outside, true);
      window.removeEventListener("resize", scroll);
      if (
        previous instanceof HTMLElement &&
        previous.isConnected &&
        (!document.activeElement?.closest(".context-menu") ||
          element?.contains(document.activeElement))
      )
        previous.focus();
    };
  }, [close]);
  return createPortal(
    <div
      ref={ref}
      className="context-menu"
      role="menu"
      aria-label={menu.title}
      style={position}
      onContextMenu={(e) => e.preventDefault()}
      onKeyDown={(e) => {
        const buttons = Array.from(
          ref.current!.querySelectorAll<HTMLButtonElement>(
            "button:not(:disabled)",
          ),
        );
        const index = buttons.indexOf(
          document.activeElement as HTMLButtonElement,
        );
        if (e.key === "Escape" || e.key === "Tab") {
          e.preventDefault();
          close();
        }
        if (["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) {
          e.preventDefault();
          const next =
            e.key === "Home"
              ? 0
              : e.key === "End"
                ? buttons.length - 1
                : (index + (e.key === "ArrowDown" ? 1 : -1) + buttons.length) %
                  buttons.length;
          buttons[next]?.focus();
        }
      }}
    >
      <div className="context-menu-title">{menu.title}</div>
      {menu.items.map((item) => (
        <button
          key={item.label}
          role="menuitem"
          disabled={item.disabled}
          className={item.danger ? "danger" : ""}
          onClick={() => {
            close();
            item.run();
          }}
        >
          {item.label}
        </button>
      ))}
    </div>,
    document.body,
  );
}
