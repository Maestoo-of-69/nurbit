// One astronaut's medical record for a specialist, in three tabs:
// Summary (automated assessment + trends + baseline reviews),
// Check-ins (answers + annotations), Notes & decisions (human judgment).
import { useState } from "react";
import { memberRecord, type CrewMember } from "../../api";
import { LevelBadge } from "../../components/LevelBadge";
import { ScreenHeader } from "../../components/ScreenHeader";
import { useCommand } from "../../useCommand";
import { RecordCheckins, RecordNotes, RecordSummary } from "./RecordParts";

type Tab = "summary" | "checkins" | "notes";

type Props = { actor: CrewMember; memberId: string; onBack: () => void; onChanged: () => void };

export function MemberRecordScreen({ actor, memberId, onBack, onChanged }: Props) {
  const [tab, setTab] = useState<Tab>("summary");
  const [version, setVersion] = useState(0);
  const record = useCommand(() => memberRecord(actor.id, memberId), [actor.id, memberId, version]);
  const reload = () => {
    setVersion((v) => v + 1);
    onChanged();
  };
  const r = record.data;

  return (
    <>
      <button type="button" className="button-link back-link" onClick={onBack}>
        ← Crew
      </button>
      <ScreenHeader
        title={r ? r.member.display_name : "Medical record"}
        subtitle={r ? `${r.member.position} · synthetic profile` : undefined}
        actions={r && <LevelBadge level={r.overall} />}
      />
      {record.error && <p className="error">{record.error}</p>}

      <div className="risk-tabs" role="tablist" aria-label="Record sections">
        {(
          [
            ["summary", "Summary"],
            ["checkins", "Check-ins"],
            ["notes", "Notes & decisions"],
          ] as [Tab, string][]
        ).map(([id, label]) => (
          <button key={id} type="button" role="tab" aria-selected={tab === id} className="risk-tab" onClick={() => setTab(id)}>
            {label}
          </button>
        ))}
      </div>

      {r && tab === "summary" && <RecordSummary actor={actor} record={r} onChanged={reload} />}
      {r && tab === "checkins" && <RecordCheckins actor={actor} record={r} onChanged={reload} />}
      {r && tab === "notes" && <RecordNotes actor={actor} record={r} onChanged={reload} />}
    </>
  );
}
