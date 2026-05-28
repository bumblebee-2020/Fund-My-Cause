"use client";

import React from "react";
import { cn } from "@/lib/utils";
import { Spinner } from "./Spinner";

// ── Linear progress bar ───────────────────────────────────────────────────────

export interface ProgressBarProps {
  value?: number;
  indeterminate?: boolean;
  label?: string;
  className?: string;
  size?: "sm" | "md" | "lg";
}

export function ProgressBar({
  value,
  indeterminate = false,
  label,
  className,
  size = "md",
}: ProgressBarProps) {
  const heights = { sm: "h-1", md: "h-2", lg: "h-3" };
  const clampedValue =
    value !== undefined ? Math.min(100, Math.max(0, value)) : 0;

  return (
    <div
      className={cn("w-full", className)}
      role="progressbar"
      aria-label={label ?? "Loading"}
      aria-valuenow={indeterminate ? undefined : clampedValue}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <div
        className={cn(
          "w-full rounded-full bg-gray-800 overflow-hidden",
          heights[size],
        )}
      >
        {indeterminate ? (
          <div className="h-full w-1/3 bg-[var(--color-brand)] rounded-full animate-[indeterminate_1.5s_ease-in-out_infinite]" />
        ) : (
          <div
            className="h-full bg-[var(--color-brand)] rounded-full transition-all duration-300 ease-out"
            style={{ width: `${clampedValue}%` }}
          />
        )}
      </div>
      {label && <p className="mt-1 text-xs text-gray-500">{label}</p>}
    </div>
  );
}

// ── Circular progress ─────────────────────────────────────────────────────────

export interface CircularProgressProps {
  value?: number;
  indeterminate?: boolean;
  size?: number;
  strokeWidth?: number;
  label?: string;
  className?: string;
}

export function CircularProgress({
  value,
  indeterminate = false,
  size = 40,
  strokeWidth = 4,
  label,
  className,
}: CircularProgressProps) {
  const r = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * r;
  const clampedValue =
    value !== undefined ? Math.min(100, Math.max(0, value)) : 0;
  const offset = circumference - (clampedValue / 100) * circumference;

  return (
    <div
      className={cn("inline-flex items-center justify-center", className)}
      role="progressbar"
      aria-label={label ?? "Loading"}
      aria-valuenow={indeterminate ? undefined : clampedValue}
    >
      <svg
        width={size}
        height={size}
        className={indeterminate ? "animate-spin" : ""}
      >
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke="currentColor"
          strokeWidth={strokeWidth}
          className="text-gray-800"
        />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke="currentColor"
          strokeWidth={strokeWidth}
          strokeDasharray={circumference}
          strokeDashoffset={indeterminate ? circumference * 0.75 : offset}
          strokeLinecap="round"
          className="text-[var(--color-brand)] transition-all duration-300"
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
      </svg>
    </div>
  );
}

// ── Loading message ───────────────────────────────────────────────────────────

export interface LoadingMessageProps {
  message?: string;
  subMessage?: string;
  progress?: number;
  indeterminate?: boolean;
  className?: string;
}

export function LoadingMessage({
  message = "Loading…",
  subMessage,
  progress,
  indeterminate = true,
  className,
}: LoadingMessageProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-center gap-4 py-12 text-center",
        className,
      )}
      role="status"
      aria-live="polite"
    >
      <Spinner size={32} label={message} />
      <div className="space-y-1">
        <p className="text-sm font-medium text-gray-300">{message}</p>
        {subMessage && <p className="text-xs text-gray-500">{subMessage}</p>}
      </div>
      {(progress !== undefined || indeterminate) && (
        <ProgressBar
          value={progress}
          indeterminate={indeterminate && progress === undefined}
          className="w-48"
          size="sm"
        />
      )}
    </div>
  );
}
