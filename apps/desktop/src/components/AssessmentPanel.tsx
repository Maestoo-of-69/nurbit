// Automated (rule-based) assessment of one risk module. Always labelled as
// machine output, kept visually apart from specialist decisions.
import type { ModuleAssessment } from "../api";
import { LevelBadge } from "./LevelBadge";

type Props = { assessment: ModuleAssessment; title: string };

export function AssessmentPanel({ assessment: a, title }: Props) {
  return (
    <section className="panel panel-wide assessment">
      <div className="panel-heading">
        <div className="panel-label">{title}</div>
        <LevelBadge level={a.level} />
      </div>
      {a.findings.length === 0 && a.level !== "no_data" && (
        <p className="panel-text">No findings in the available data.</p>
      )}
      <ul className="finding-list">
        {a.findings.map((f) => (
          <li key={f.message}>
            <LevelBadge level={f.level} />
            <span>
              {f.message}{" "}
              <span className="cell-note">
                Basis:{" "}
                {f.basis === "prototype_assumption"
                  ? "prototype assumption, needs specialist validation"
                  : `source ${f.basis}`}
              </span>
            </span>
          </li>
        ))}
      </ul>
      {a.gaps.length > 0 && (
        <p className="cell-note">Missing data: {a.gaps.map((g) => `${g.indicator} — ${g.note}`).join(" · ")}</p>
      )}
      {a.uses_simulated_data && <p className="cell-note">Uses simulated device data (synthetic, illustrative values).</p>}
    </section>
  );
}
