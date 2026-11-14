// Text + colour badges for assessment levels and baseline status.
// Colour always comes with words, never alone.
import type { BaselineStatus, Level } from "../api";
import { StatusBadge, type Status } from "./StatusBadge";

const LEVEL: Record<Level, { status: Status; label: string }> = {
  no_data: { status: "inactive", label: "No data" },
  nothing_to_report: { status: "normal", label: "Nothing to report" },
  attention: { status: "attention", label: "Needs attention" },
  specialist_review: { status: "review", label: "Specialist review" },
};

export function LevelBadge({ level }: { level: Level }) {
  const { status, label } = LEVEL[level];
  return <StatusBadge status={status} label={label} />;
}

export function BaselineBadge({ status }: { status: BaselineStatus }) {
  switch (status.state) {
    case "insufficient_data":
      return <StatusBadge status="inactive" label="Not enough history" />;
    case "within_baseline":
      return <StatusBadge status="normal" label="Within your usual range" />;
    case "temporary_deviation":
      return <StatusBadge status="attention" label={`Once ${status.direction} your usual range`} />;
    case "persistent_change":
      return <StatusBadge status="review" label={`${status.count}× ${status.direction} usual range · review`} />;
  }
}

/** Highest level first, for summaries. */
export const LEVEL_ORDER: Level[] = ["specialist_review", "attention", "nothing_to_report", "no_data"];
