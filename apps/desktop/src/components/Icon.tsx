// Small inline line icons (24x24, stroke = currentColor), so no icon library
// is needed and icons follow the text colour of their parent.
export type IconName = "home" | "check" | "chart" | "shield" | "chat" | "sync" | "info" | "users" | "inbox";

const PATHS: Record<IconName, string> = {
  home: "M4 11 12 4l8 7v8a1 1 0 0 1-1 1h-4v-6H9v6H5a1 1 0 0 1-1-1z",
  check: "M9 4h6v3H9zM7 5H5v15h14V5h-2M8.5 13l2.5 2.5 4.5-5",
  chart: "M4 4v16h16M7 15l4-5 3 3 5-6",
  shield: "M12 3 5 6v5c0 4.5 3 8 7 10 4-2 7-5.5 7-10V6zM12 9v6M9 12h6",
  chat: "M5 5h14v10H10l-4 4v-4H5z",
  sync: "M20 11a8 8 0 0 0-14-4.5L4 9M4 4v5h5M4 13a8 8 0 0 0 14 4.5l2-2.5M20 20v-5h-5",
  info: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 11v6M12 7.5v.5",
  users: "M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM3 20c0-3.3 2.7-6 6-6s6 2.7 6 6M16 4.5a3.5 3.5 0 0 1 0 6.5M18 14c2 .8 3 2.8 3 6",
  inbox: "M4 13h4l1.5 3h5L16 13h4M4 13l2.5-8h11L20 13v6H4z",
};

type Props = { name: IconName; size?: number };

export function Icon({ name, size = 20 }: Props) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={PATHS[name]} />
    </svg>
  );
}
