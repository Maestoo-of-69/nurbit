// Specialist home: the whole crew at a glance, highest concern first.
// Opening a row shows that astronaut's medical record.
import { useState } from "react";
import { crewOverview, type CrewMember, type CrewSummary, type Level } from "../../api";
import { LEVEL_ORDER, LevelBadge } from "../../components/LevelBadge";
import { ScreenHeader } from "../../components/ScreenHeader";
import { useCommand } from "../../useCommand";
import { MemberRecordScreen } from "./MemberRecordScreen";

type Props = { actor: CrewMember; refreshKey: number; onChanged: () => void };

const rank = (l: Level) => LEVEL_ORDER.indexOf(l);

export function CrewScreen({ actor, refreshKey, onChanged }: Props) {
  const [openId, setOpenId] = useState<string | null>(null);
  const crew = useCommand(() => crewOverview(actor.id), [actor.id, refreshKey, openId]);

  if (openId) {
    return <MemberRecordScreen actor={actor} memberId={openId} onBack={() => setOpenId(null)} onChanged={onChanged} />;
  }

  const rows = [...(crew.data ?? [])].sort((a, b) => rank(a.overall) - rank(b.overall));

  return (
    <>
      <ScreenHeader title="Crew" subtitle="Automated status per astronaut (prototype rules). Your decisions are recorded separately." />
      {crew.error && <p className="error">{crew.error}</p>}
      <section className="panel panel-wide table-panel">
        <table className="history-table crew-table">
          <thead>
            <tr>
              <th>Astronaut</th>
              <th>Automated status</th>
              <th>Last check-in</th>
              <th>What needs attention</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <CrewRow key={row.member.id} row={row} onOpen={() => setOpenId(row.member.id)} />
            ))}
          </tbody>
        </table>
      </section>
    </>
  );
}

function CrewRow({ row, onOpen }: { row: CrewSummary; onOpen: () => void }) {
  const last = row.last_checkin_at
    ? new Date(row.last_checkin_at).toLocaleString("en-GB", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" })
    : "—";
  return (
    <tr>
      <td>
        <div className="profile-name">{row.member.display_name}</div>
        <div className="cell-note">{row.member.position}</div>
      </td>
      <td>
        <LevelBadge level={row.overall} />
        {row.pending_second_reviews > 0 && <div className="cell-note">{row.pending_second_reviews} decision(s) awaiting second review</div>}
      </td>
      <td>
        {last}
        {!row.checked_in_today && <div className="cell-note">No check-in in the last 24 h</div>}
      </td>
      <td className="flags-cell">
        {row.flags.length === 0 ? <span className="cell-note">Nothing flagged</span> : row.flags.map((f) => <div key={f}>{f}</div>)}
      </td>
      <td>
        <button type="button" className="button-secondary" onClick={onOpen}>
          Open record
        </button>
      </td>
    </tr>
  );
}
