/** Screens listed under "Settings" in the sidebar, in display order. */
export const SETTINGS_SCREENS = [
  "languages",
  "providers",
  "privacy",
  "context",
  "appearance",
  "rtl",
  "about",
] as const;

export type ScreenId = "live" | (typeof SETTINGS_SCREENS)[number];
