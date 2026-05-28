"use client";

import React from "react";
import { Sun, Moon } from "lucide-react";
import { useTheme } from "@/context/ThemeContext";
import { cn } from "@/lib/utils";

interface ThemeToggleProps {
  className?: string;
  showLabel?: boolean;
}

export function ThemeToggle({
  className,
  showLabel = false,
}: ThemeToggleProps) {
  const { theme, toggleTheme } = useTheme();
  const isDark = theme === "dark";

  return (
    <button
      onClick={toggleTheme}
      aria-label={isDark ? "Switch to light mode" : "Switch to dark mode"}
      aria-pressed={isDark}
      className={cn(
        "flex items-center gap-2 p-2 rounded-[var(--radius-xl)]",
        "bg-[var(--color-surface-elevated)] hover:bg-[var(--color-border-subtle)]",
        "text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]",
        "transition-colors duration-200",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-brand)] focus-visible:ring-offset-2",
        className,
      )}
    >
      {isDark ? (
        <Sun
          size={18}
          className="text-[var(--color-warning)]"
          aria-hidden="true"
        />
      ) : (
        <Moon
          size={18}
          className="text-[var(--color-brand)]"
          aria-hidden="true"
        />
      )}
      {showLabel && (
        <span className="text-sm">{isDark ? "Light mode" : "Dark mode"}</span>
      )}
    </button>
  );
}
