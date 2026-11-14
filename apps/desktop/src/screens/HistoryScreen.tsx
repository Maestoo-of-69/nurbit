// History: trend per daily rating against the person's own baseline,
// cross-indicator patterns (labelled as observations, never causes),
// and the full table of check-ins (the text view of the same data).
import {
  dailyScore,
  getMemberTrends,
  getRiskAssessments,
  listCheckins,
  type CheckInRecord,
  type CrewMember,
  type CrossRiskPattern,
  type EvidenceLevel,
} from "../api";
import { EmptyState } from "../components/EmptyState";
import { BaselineBadge } from "../components/LevelBadge";
import { ScreenHeader } from "../components/ScreenHeader";
import { TrendChart } from "../components/TrendChart";
import { useCommand } from "../useCommand";

const COLUMNS = [
  { id: "daily.rested", label: "Rested" },
  { id: "daily.physical", label: "Body" },
  { id: "daily.mood_energy", label: "Mood & energy" },
];

type Props = { member: CrewMember; refreshKey: number };

export function HistoryScreen({ member, refreshKey }: Props) {
  const deps = [member.id, refreshKey];
  const checkins = useCommand(() => listCheckins(member.id), deps);
  const trends = useCommand(() => getMemberTrends(member.id), deps);
  const assessments = useCommand(() => getRiskAssessments(member.id), deps);
  const rows = checkins.data ?? [];

  if (member.role !== "astronaut") {
    return (
      <>
        <ScreenHeader title="History" />
        <div className="callout">Specialists do not have their own check-in history. Crew review: stage 9.</div>
      </>
    );
  }

  return (
    <>
      <ScreenHeader title="History" subtitle={`${member.display_name} · ${rows.length} check-ins`} />
      {(checkins.error ?? trends.error) && <p className="error">{checkins.error ?? trends.error}</p>}
      {checkins.data && rows.length === 0 && (
        <EmptyState icon="chart" title="No check-ins yet" text="Saved check-ins will appear here." stage={6} />
      )}

      {trends.data && rows.length > 0 && (
        <>
          <div className="trend-grid">
            {trends.data.map((s) => (
              <section key={s.question_id} className="panel trend-card">
                <div className="trend-head">
                  <h2>{s.label}</h2>
                  <BaselineBadge status={s.trend.status} />
                </div>
                <p className="cell-note">
                  {s.trend.baseline
                    ? `Usual range ${Math.max(0, s.trend.baseline.low).toFixed(1)}–${Math.min(10, s.trend.baseline.high).toFixed(1)} (your average ${s.trend.baseline.mean.toFixed(1)} from ${s.trend.baseline.values_used} earlier check-ins).`
                    : "Your usual range appears after enough check-ins."}
                </p>
                <TrendChart label={s.label} points={s.points} baseline={s.trend.baseline} />
              </section>
            ))}
          </div>
          <p className="cell-note chart-key">
            Grey band: your usual range (own history, not a medical norm). Filled dots: your answers; hollow
            dots: synthetic demo history. Band width and windows are prototype assumptions; a baseline is
            never changed automatically.
          </p>
        </>
      )}

      {assessments.data && rows.length > 0 && <Patterns patterns={assessments.data.patterns} />}

      {rows.length > 0 && (
        <section className="panel panel-wide table-panel">
          <table className="history-table">
            <thead>
              <tr>
                <th>Date</th>
                {COLUMNS.map((c) => (
                  <th key={c.id}>{c.label}</th>
                ))}
                <th>Follow-ups</th>
                <th>Source</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => (
                <HistoryRow key={r.id} record={r} />
              ))}
            </tbody>
          </table>
        </section>
      )}
    </>
  );
}

const EVIDENCE_LABEL: Record<EvidenceLevel, string> = {
  observed_correlation: "Observed correlation",
  possible_association: "Possible association",
  established_relationship: "Established relationship",
  unverified_hypothesis: "Unverified hypothesis",
};

function describeR(r: number) {
  const direction = r >= 0 ? "move together" : "move in opposite directions";
  return `r = ${r.toFixed(2)}: in your data these ratings ${direction}`;
}

function Patterns({ patterns }: { patterns: CrossRiskPattern[] }) {
  return (
    <section className="panel panel-wide">
      <div className="panel-label">Patterns between indicators</div>
      <p className="panel-text">Correlation is not proof of cause. These are observations in your own data.</p>
      <ul className="pattern-list">
        {patterns.map((p) => (
          <li key={`${p.first}-${p.second}`}>
            <div className="pattern-title">
              {p.first} ↔ {p.second}
            </div>
            <div className="pattern-row">
              <span className="class-tag class-derived">{EVIDENCE_LABEL[p.in_your_data]}</span>
              {p.r === null
                ? `Not enough varied data (${p.pairs} check-ins).`
                : `${describeR(p.r)} (${p.pairs} check-ins).`}
            </div>
            <div className="pattern-row">
              <span className="class-tag class-review">{EVIDENCE_LABEL[p.literature]}</span>
              {p.literature_note} {p.literature_source && <span className="mono">[{p.literature_source}]</span>}
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}

function HistoryRow({ record }: { record: CheckInRecord }) {
  const followUps = record.answers.filter((a) => a.triggered_by_rule).length;
  const date = new Date(record.completed_at).toLocaleString("en-GB", {
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
  return (
    <tr>
      <td>{date}</td>
      {COLUMNS.map((c) => (
        <td key={c.id}>
          <ScoreCell value={dailyScore(record, c.id)} />
        </td>
      ))}
      <td>{followUps > 0 ? followUps : "—"}</td>
      <td>
        <span className={`origin origin-${record.origin}`}>
          {record.origin === "simulated" ? "Synthetic" : "Self-report"}
        </span>
      </td>
    </tr>
  );
}

function ScoreCell({ value }: { value: number | null }) {
  if (value === null) return <span>—</span>;
  return (
    <span className="score">
      <span className="score-bar" style={{ width: `${value * 10}%` }} aria-hidden="true" />
      <span className="score-value">{value}</span>
    </span>
  );
}
