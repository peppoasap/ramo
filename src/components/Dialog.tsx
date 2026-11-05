import { useEffect, useRef, type ReactNode } from "react";
import { X } from "@phosphor-icons/react";
export function Dialog({
  title,
  children,
  onClose,
  locked = false,
  wide = false,
  className = "",
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  locked?: boolean;
  wide?: boolean;
  className?: string;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    ref.current?.showModal();
    const dialog = ref.current;
    return () => dialog?.close();
  }, []);
  return (
    <dialog
      ref={ref}
      className={`${wide ? "wide" : ""} ${className}`.trim()}
      onCancel={(e) => {
        e.preventDefault();
        if (!locked) onClose();
      }}
    >
      <div className="dialog-heading">
        <h2>{title}</h2>
        <button
          className="icon"
          aria-label="Close dialog"
          disabled={locked}
          onClick={onClose}
        >
          <X size={20} />
        </button>
      </div>
      {children}
    </dialog>
  );
}
