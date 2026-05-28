"use client";

import React from "react";

interface SkipNavProps {
  contentId?: string;
}

export function SkipNav({ contentId = "main-content" }: SkipNavProps) {
  return (
    <a
      href={`#${contentId}`}
      className="sr-only focus:not-sr-only focus:fixed focus:top-4 focus:left-4 focus:z-[9999] focus:px-4 focus:py-2 focus:bg-[var(--color-brand)] focus:text-white focus:rounded-[var(--radius-md)] focus:font-medium focus:outline-none focus:shadow-lg"
    >
      Skip to main content
    </a>
  );
}
