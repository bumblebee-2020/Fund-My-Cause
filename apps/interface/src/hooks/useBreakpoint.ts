"use client";

import { useEffect, useState } from "react";

const breakpoints = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  "2xl": 1536,
} as const;

type Breakpoint = keyof typeof breakpoints;

function getMatches(): Record<Breakpoint, boolean> {
  if (typeof window === "undefined") {
    return { sm: false, md: false, lg: false, xl: false, "2xl": false };
  }
  return Object.fromEntries(
    Object.entries(breakpoints).map(([key, px]) => [
      key,
      window.innerWidth >= px,
    ]),
  ) as Record<Breakpoint, boolean>;
}

export function useBreakpoint() {
  const [matches, setMatches] =
    useState<Record<Breakpoint, boolean>>(getMatches);

  useEffect(() => {
    const queries = Object.entries(breakpoints).map(([_key, px]) => {
      const mq = window.matchMedia(`(min-width: ${px}px)`);
      const handler = () => setMatches(getMatches());
      mq.addEventListener("change", handler);
      return () => mq.removeEventListener("change", handler);
    });
    return () => queries.forEach((cleanup) => cleanup());
  }, []);

  return {
    ...matches,
    isMobile: !matches.md,
    isTablet: matches.md && !matches.lg,
    isDesktop: matches.lg,
  };
}
