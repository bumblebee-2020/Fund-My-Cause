"use client";

import React, {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
} from "react";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

interface ModalConfig {
  id: string;
  title?: string;
  content: React.ReactNode;
  footer?: React.ReactNode;
  size?: "sm" | "md" | "lg" | "xl";
  closeOnBackdropClick?: boolean;
  onClose?: () => void;
}

interface ModalContextType {
  openModal: (config: Omit<ModalConfig, "id">) => string;
  closeModal: (id: string) => void;
  closeAll: () => void;
}

const ModalContext = createContext<ModalContextType | undefined>(undefined);

const sizes: Record<NonNullable<ModalConfig["size"]>, string> = {
  sm: "max-w-sm",
  md: "max-w-md",
  lg: "max-w-lg",
  xl: "max-w-xl",
};

function ModalItem({
  modal,
  index,
  total,
  onClose,
}: {
  modal: ModalConfig;
  index: number;
  total: number;
  onClose: () => void;
}) {
  const isTop = index === total - 1;
  const offset = (total - 1 - index) * 8;

  return (
    <div
      className={cn(
        "fixed inset-0 z-50 flex items-center justify-center",
        "animate-in fade-in duration-200",
      )}
      style={{ zIndex: 50 + index }}
      role="presentation"
    >
      <div
        className={cn(
          "absolute inset-0 bg-black/50 transition-opacity duration-200",
          isTop ? "opacity-100" : "opacity-0 pointer-events-none",
        )}
        onClick={() => modal.closeOnBackdropClick !== false && onClose()}
        aria-hidden="true"
      />
      <div
        className={cn(
          "relative bg-[var(--color-surface)] rounded-[var(--radius-2xl)] shadow-2xl w-full mx-4",
          "border border-[var(--color-border)]",
          "transition-all duration-200",
          "animate-in zoom-in-95 fade-in duration-200",
          sizes[modal.size ?? "md"],
        )}
        style={{
          transform: `translateY(-${offset}px) scale(${1 - (total - 1 - index) * 0.02})`,
        }}
        role="dialog"
        aria-modal="true"
        aria-labelledby={modal.title ? `modal-title-${modal.id}` : undefined}
      >
        <div className="flex items-center justify-between p-6 border-b border-[var(--color-border)]">
          {modal.title && (
            <h2
              id={`modal-title-${modal.id}`}
              className="text-lg font-semibold text-[var(--color-text-primary)]"
            >
              {modal.title}
            </h2>
          )}
          <button
            onClick={onClose}
            className="ml-auto p-1 rounded-[var(--radius-lg)] hover:bg-[var(--color-surface-elevated)] transition-colors text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
            aria-label="Close modal"
          >
            <X className="w-5 h-5" aria-hidden="true" />
          </button>
        </div>
        <div className="p-6 text-[var(--color-text-primary)]">
          {modal.content}
        </div>
        {modal.footer && (
          <div className="flex items-center justify-end gap-3 p-6 border-t border-[var(--color-border)]">
            {modal.footer}
          </div>
        )}
      </div>
    </div>
  );
}

export function ModalProvider({ children }: { children: React.ReactNode }) {
  const [stack, setStack] = useState<ModalConfig[]>([]);
  const counterRef = useRef(0);

  const openModal = useCallback((config: Omit<ModalConfig, "id">) => {
    const id = `modal-${++counterRef.current}`;
    setStack((prev) => [...prev, { ...config, id }]);
    return id;
  }, []);

  const closeModal = useCallback((id: string) => {
    setStack((prev) => {
      const modal = prev.find((m) => m.id === id);
      modal?.onClose?.();
      return prev.filter((m) => m.id !== id);
    });
  }, []);

  const closeAll = useCallback(() => {
    setStack((prev) => {
      prev.forEach((m) => m.onClose?.());
      return [];
    });
  }, []);

  return (
    <ModalContext.Provider value={{ openModal, closeModal, closeAll }}>
      {children}
      {stack.map((modal, index) => (
        <ModalItem
          key={modal.id}
          modal={modal}
          index={index}
          total={stack.length}
          onClose={() => closeModal(modal.id)}
        />
      ))}
    </ModalContext.Provider>
  );
}

export function useModal() {
  const ctx = useContext(ModalContext);
  if (!ctx) throw new Error("useModal must be used within a ModalProvider");
  return ctx;
}
