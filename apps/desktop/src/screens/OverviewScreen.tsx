// Home screen: today's check, monitoring status, notifications, risk modules
// and the assistant. Values come from the local database; nothing invented.
import {
  getRiskAssessments,
  getRiskModules,
  getSyncStatus,
  listCheckins,
  type CheckInRecord,
  type CrewMember,
} from "../api";
import { Icon } from "../components/Icon";
import { LEVEL_ORDER, LevelBadge } from "../components/LevelBadge";
import { ScreenHeader } from "../components/ScreenHeader";
import { StatusBadge } from "../components/StatusBadge";
import type { Screen } from "../navigation";
import { useCommand } from "../useCommand";

type Props = { member: CrewMember; refreshKey: number; onNavigate: (screen: Screen) => void };

function isToday(ms: number) {
  return new Date(ms).toDateString() === new Date().toDateString();
}

/** Today's own (not simulated) check-in, if one was saved. */
function todaysCheckin(records: CheckInRecord[] | null) {
  return records?.find((r) => r.origin === "self_report" && isToday(r.completed_at)) ?? null;
}

export function OverviewScreen({ member, refreshKey, onNavigate }: Props) {
  const risks = useCommand(getRiskModules);
  const checkins = useCommand(() => listCheckins(member.id), [member.id, refreshKey]);
  const sync = useCommand(getSyncStatus, [refreshKey]);
  const today = todaysCheckin(checkins.data);
  const isAstronaut = member.role === "astronaut";
  const assessments = useCommand(
    () => (isAstronaut ? getRiskAssessments(member.id) : Promise.resolve(null)),
    [member.id, refreshKey],
  );
  const levelOf = (id: string) => assessments.data?.modules.find((m) => m.module_id === id)?.level;
  const overall = LEVEL_ORDER.find(
    (l) => assessments.data?.modules.some((m) => m.level === l) || assessments.data?.safety.some((f) => f.level === l),
  );
  // Fixed English locale: the interface language is English for now.
  const date = new Date().toLocaleDateString("en-GB", { weekday: "long", day: "numeric", month: "long" });

  return (
    <>
      <ScreenHeader title={`Hello, ${member.display_name}`} subtitle={date} />

      <div className="overview-grid">
        <section className={`panel ${today ? "panel-done" : "panel-primary"}`}>
          <div className="panel-label">Today's check</div>
          {isAstronaut ? (
            <>
              <div className="panel-value">{today ? "Completed" : "Not completed"}</div>
              <p className="panel-text">
                {today
                  ? `Saved at ${new Date(today.completed_at).toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" })}.`
                  : "Three questions, about one minute."}
              </p>
              {!today && (
                <button type="button" className="button-primary" onClick={() => onNavigate("daily-check")}>
                  Open daily check
                </button>
              )}
            </>
          ) : (
            <p className="panel-text">Specialists do not complete daily checks. Specialist view: stage 9.</p>
          )}
        </section>

        <section className="panel">
          <div className="panel-label">Monitoring status</div>
          {overall ? <LevelBadge level={overall} /> : <StatusBadge status="inactive" label="Not evaluated" />}
          <p className="panel-text">
            {isAstronaut
              ? "Highest level across the five risk modules. Not a diagnosis."
              : "Crew-level review arrives with the specialist view (stage 9)."}
          </p>
        </section>

        <section className="panel">
          <div className="panel-label">Waiting to send</div>
          <div className={`panel-value ${sync.data?.pending ? "" : "panel-value-quiet"}`}>
            {sync.data?.pending ?? "—"}
          </div>
          <p className="panel-text">Records queued for the medical team. Details on the Sync screen.</p>
        </section>
      </div>

      <section className="panel panel-wide">
        <div className="panel-heading">
          <div className="panel-label">Risk modules</div>
          <button type="button" className="button-link" onClick={() => onNavigate("risks")}>
            View all
          </button>
        </div>
        {risks.error && <p className="error">Could not load risk modules: {risks.error}</p>}
        <ul className="risk-rows">
          {risks.data?.map((module) => (
            <li key={module.id}>
              <span className="risk-name">{module.short_title}</span>
              {levelOf(module.id) ? (
                <LevelBadge level={levelOf(module.id)!} />
              ) : (
                <StatusBadge status="inactive" label="Not evaluated" />
              )}
            </li>
          ))}
        </ul>
      </section>

      <section className="panel panel-wide panel-row">
        <div className="panel-row-icon">
          <Icon name="chat" size={22} />
        </div>
        <div>
          <div className="panel-label">Assistant</div>
          <p className="panel-text">
            Ask about a symptom and get excerpts from verified sources. It never diagnoses or prescribes.
          </p>
        </div>
        <button type="button" className="button-secondary" onClick={() => onNavigate("assistant")}>
          Open
        </button>
      </section>
    </>
  );
}
