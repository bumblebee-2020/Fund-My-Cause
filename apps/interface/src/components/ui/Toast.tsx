"use client";

import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  ReactNode,
} from "react";
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from "lucide-react";

export type ToastType = "success" | "error" | "info" | "warning";

const TOAST_MAX = 5;

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  txHash?: string;
}

interface ToastContextType {
  toasts: Toast[];
  addToast: (message: string, type: ToastType, txHash?: string) => void;
  removeToast: (id: string) => void;
}

const ToastContext = createContext<ToastContextType | null>(null);

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback(
    (message: string, type: ToastType, txHash?: string) => {
      const id = Math.random().toString(36).substring(2, 9);
      setToasts((prev) => {
        const next = [...prev, { id, message, type, txHash }];
        return next.length > TOAST_MAX
          ? next.slice(next.length - TOAST_MAX)
          : next;
      });

      // Auto-dismiss success/info/warning after 5 seconds
      if (type !== "error") {
        setTimeout(() => {
          setToasts((prev) => prev.filter((t) => t.id !== id));
        }, 5000);
      }
    },
    [],
  );

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ toasts, addToast, removeToast }}>
      {children}
      <ToastContainer toasts={toasts} removeToast={removeToast} />
    </ToastContext.Provider>
  );
}

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be used within ToastProvider");
  return ctx;
}

function ToastContainer({
  toasts,
  removeToast,
}: {
  toasts: Toast[];
  removeToast: (id: string) => void;
}) {
  if (toasts.length === 0) return null;

  return (
    <div
      className="fixed bottom-4 right-4 z-50 flex flex-col gap-2"
      role="status"
      aria-live="polite"
      aria-label="Notifications"
    >
      {toasts.map((toast) => (
        <ToastItem
          key={toast.id}
          toast={toast}
          onClose={() => removeToast(toast.id)}
        />
      ))}
    </div>
  );
}

function ToastItem({ toast, onClose }: { toast: Toast; onClose: () => void }) {
  const icons = {
    success: <CheckCircle className="text-green-400" size={20} />,
    error: <AlertCircle className="text-red-400" size={20} />,
    info: <Info className="text-blue-400" size={20} />,
    warning: <AlertTriangle className="text-yellow-400" size={20} />,
  };

  const borders = {
    success: "border-green-400/30",
    error: "border-red-400/30",
    info: "border-blue-400/30",
    warning: "border-yellow-400/30",
  };

  return (
    <div
      className={`flex items-start gap-3 p-4 bg-[var(--color-surface)] rounded-lg border ${borders[toast.type]} shadow-lg min-w-[300px] max-w-md animate-in slide-in-from-right`}
    >
      {icons[toast.type]}
      <div className="flex-1">
        <p className="text-sm text-[var(--color-text-primary)]">
          {toast.message}
        </p>
        {toast.txHash && (
          <a
            href={`https://stellar.expert/explorer/testnet/tx/${toast.txHash}`}
            target="_blank"
            rel="noopener noreferrer"
            aria-label="View transaction on Stellar Expert (opens in new tab)"
            className="text-xs text-[var(--color-brand)] hover:underline mt-1 inline-block"
          >
            View on Stellar Expert →
          </a>
        )}
      </div>
      <button
        onClick={onClose}
        aria-label="Dismiss notification"
        className="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
      >
        <X size={16} />
      </button>
    </div>
  );
}
