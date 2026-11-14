// App navigation without a router: a desktop app has no address bar, so the
// open screen is plain React state. Adding a screen = add it here + in App.tsx.
import type { IconName } from "./components/Icon";

export type Screen =
  | "crew"
  | "pending-reviews"
  | "overview"
  | "daily-check"
  | "history"
  | "risks"
  | "assistant"
  | "sync"
  | "about";

export interface NavItem {
  screen: Screen;
  label: string;
  icon: IconName;
}

export interface NavGroup {
  title: string;
  items: NavItem[];
}

const SPECIALIST_GROUPS: NavGroup[] = [
  {
    title: "Medical team",
    items: [
      { screen: "crew", label: "Crew", icon: "users" },
      { screen: "pending-reviews", label: "Pending reviews", icon: "inbox" },
      { screen: "risks", label: "Risk modules", icon: "shield" },
    ],
  },
  {
    title: "System",
    items: [
      { screen: "sync", label: "Sync", icon: "sync" },
      { screen: "about", label: "About", icon: "info" },
    ],
  },
];

/** Menu for the signed-in role; Rust still checks permissions on every action. */
export function navGroupsFor(role: "astronaut" | "specialist"): NavGroup[] {
  return role === "specialist" ? SPECIALIST_GROUPS : NAV_GROUPS;
}

export function homeScreenFor(role: "astronaut" | "specialist"): Screen {
  return role === "specialist" ? "crew" : "overview";
}

const NAV_GROUPS: NavGroup[] = [
  {
    title: "Monitoring",
    items: [
      { screen: "overview", label: "Overview", icon: "home" },
      { screen: "daily-check", label: "Daily check", icon: "check" },
      { screen: "history", label: "History", icon: "chart" },
      { screen: "risks", label: "Risk modules", icon: "shield" },
      { screen: "assistant", label: "Assistant", icon: "chat" },
    ],
  },
  {
    title: "System",
    items: [
      { screen: "sync", label: "Sync", icon: "sync" },
      { screen: "about", label: "About", icon: "info" },
    ],
  },
];
