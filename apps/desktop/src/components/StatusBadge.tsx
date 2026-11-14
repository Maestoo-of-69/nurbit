// One badge style per monitoring state. The colours carry meaning and are
// always paired with text, so the state never depends on colour alone.
export type Status = "normal" | "attention" | "alert" | "review" | "inactive";

const LABELS: Record<Status, string> = {
  normal: "Normal",
  attention: "Needs attention",
  alert: "High priority",
  review: "Awaiting specialist",
  inactive: "Not active yet",
};

type Props = { status: Status; label?: string };

export function StatusBadge({ status, label }: Props) {
  return <span className={`badge badge-${status}`}>{label ?? LABELS[status]}</span>;
}
