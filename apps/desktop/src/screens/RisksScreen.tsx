// The five approved risk modules, as configured in Rust (medical-core).
// For each: what is monitored, every indicator with its class and whether it
// exists in flight, official limits with their source, and limitations.
import { useState } from "react";
import {
  getRiskAssessments,
  getRiskModules,
  type BetaSource,
  type CrewMember,
  type HrpCategory,
  type IndicatorClass,
  type RiskModuleInfo,
} from "../api";
import { AssessmentPanel } from "../components/AssessmentPanel";
import { ScreenHeader } from "../components/ScreenHeader";
import { StatusBadge } from "../components/StatusBadge";
import { useCommand } from "../useCommand";

const CATEGORY_LABEL: Record<HrpCategory, string> = {
  human_physiology: "Human physiology",
  behavioral_performance: "Behavioral performance",
  space_radiation: "Space radiation",
};

const CLASS_LABEL: Record<IndicatorClass, string> = {
  directly_measured: "Measured",
  self_reported: "Self-reported",
  derived: "Derived",
  risk_estimate: "Risk estimate",
  specialist_finding: "Specialist finding",
};

const SOURCE_LABEL: Record<BetaSource, string> = {
  daily_questionnaire: "Daily check",
  follow_up_questionnaire: "Follow-up questions",
  manual_entry: "Manual entry",
  simulated_device: "Simulated device",
  not_in_beta: "Not in beta",
};

type Props = { member: CrewMember; refreshKey: number };

export function RisksScreen({ member, refreshKey }: Props) {
  const modules = useCommand(getRiskModules);
  const assessments = useCommand(
    () => (member.role === "astronaut" ? getRiskAssessments(member.id) : Promise.resolve(null)),
    [member.id, refreshKey],
  );
  const [openId, setOpenId] = useState<string>("bone");
  const open = modules.data?.find((m) => m.id === openId) ?? modules.data?.[0];
  const assessment = assessments.data?.modules.find((a) => a.module_id === open?.id) ?? null;

  return (
    <>
      <ScreenHeader
        title="Risk modules"
        subtitle="Five approved NASA Human Research Program risks, one shared monitoring framework."
      />
      <div className="callout">
        Monitoring is not diagnosis. Published sources give no in-flight alert thresholds for most of
        these indicators; rules marked "prototype assumption" need specialist validation.
      </div>
      {modules.error && <p className="error">Could not load risk modules: {modules.error}</p>}

      <div className="risk-tabs" role="tablist" aria-label="Risk modules">
        {modules.data?.map((m) => (
          <button
            key={m.id}
            type="button"
            role="tab"
            aria-selected={m.id === open?.id}
            className="risk-tab"
            onClick={() => setOpenId(m.id)}
          >
            {m.short_title}
            {assessments.data && (
              <span className={`tab-dot tab-dot-${assessments.data.modules.find((a) => a.module_id === m.id)?.level}`} aria-hidden="true" />
            )}
          </button>
        ))}
      </div>

      {assessment && (
        <AssessmentPanel assessment={assessment} title={`Automated assessment · ${member.display_name}`} />
      )}
      {open && <ModuleDetail module={open} />}
    </>
  );
}

function ModuleDetail({ module: m }: { module: RiskModuleInfo }) {
  return (
    <article className="risk-detail" role="tabpanel">
      <header className="risk-card-header">
        <span className="risk-category">{CATEGORY_LABEL[m.category]}</span>
        <StatusBadge status="inactive" label="Prototype rules · not clinically validated" />
      </header>
      <h2>{m.hrp_title}</h2>
      <p className="risk-monitored">{m.monitored}</p>

      <table className="history-table indicator-table">
        <thead>
          <tr>
            <th>Indicator</th>
            <th>Type</th>
            <th>In flight</th>
            <th>vs. own baseline</th>
            <th>In this beta</th>
          </tr>
        </thead>
        <tbody>
          {m.indicators.map((i) => (
            <tr key={i.name}>
              <td>
                {i.name}
                {i.equipment && <div className="cell-note">{i.equipment}</div>}
              </td>
              <td>
                <span className={`class-tag class-${i.class}`}>{CLASS_LABEL[i.class]}</span>
              </td>
              <td>{i.available_in_flight ? "Yes" : "No"}</td>
              <td>{i.baseline_comparable ? "Yes" : "—"}</td>
              <td>{SOURCE_LABEL[i.beta_source]}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div className="risk-columns">
        {m.reference_limits.length > 0 && (
          <section>
            <h3>Official limits</h3>
            {m.reference_limits.map((l) => (
              <p key={l.description}>
                {l.description} <span className="mono">[{l.source_id}]</span>
                <span className="cell-note">{l.applies}</span>
              </p>
            ))}
          </section>
        )}
        <section>
          <h3>Extra questions when</h3>
          {m.questionnaire_triggers.length > 0 ? (
            <ul className="plain-list">
              {m.questionnaire_triggers.map((t) => (
                <li key={t}>{t}</li>
              ))}
            </ul>
          ) : (
            <p className="cell-note">No questionnaire trigger: this risk is followed through measurements.</p>
          )}
        </section>
        <section>
          <h3>Sent to specialists</h3>
          <ul className="plain-list">
            {m.specialist_package.map((s) => (
              <li key={s}>{s}</li>
            ))}
          </ul>
        </section>
        <section>
          <h3>Limitations</h3>
          <ul className="plain-list">
            {m.limitations.map((s) => (
              <li key={s}>{s}</li>
            ))}
          </ul>
        </section>
      </div>
      <p className="cell-note">
        Sources: <span className="mono">{m.reference_ids.join(", ")}</span> (see References in the docs)
      </p>
    </article>
  );
}
