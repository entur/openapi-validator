import { useEffect, useState } from "react";

export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() => window.matchMedia(query).matches);
  useEffect(() => {
    const mql = window.matchMedia(query);
    const onChange = (e: MediaQueryListEvent) => setMatches(e.matches);
    setMatches(mql.matches);
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, [query]);
  return matches;
}

/**
 * Follow the OS color scheme and mirror it to `data-color-mode` on <html>,
 * which switches all Entur token values between light and dark.
 */
export function useColorMode(): "light" | "dark" {
  const dark = useMediaQuery("(prefers-color-scheme: dark)");
  const mode = dark ? "dark" : "light";
  useEffect(() => {
    document.documentElement.setAttribute("data-color-mode", mode);
  }, [mode]);
  return mode;
}
