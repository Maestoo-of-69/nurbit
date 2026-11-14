// First screen: who is using Nurbit on this computer right now.
// Beta: no password yet (synthetic profiles only); see docs/DEVELOPMENT_PLAN.md.
import { listCrew, type CrewMember } from "../api";
import brandMark from "../assets/brand-mark.svg";
import { useCommand } from "../useCommand";

type Props = { onSelect: (member: CrewMember) => void };

export function ProfilePicker({ onSelect }: Props) {
  const crew = useCommand(listCrew);
  const astronauts = crew.data?.filter((m) => m.role === "astronaut") ?? [];
  const specialists = crew.data?.filter((m) => m.role === "specialist") ?? [];

  return (
    <div className="picker">
      <div className="picker-card">
        <div className="picker-brand">
          <img src={brandMark} alt="" width={40} height={40} />
          <div>
            <h1>Nurbit</h1>
            <p>Crew health check-ins, on board and offline.</p>
          </div>
        </div>
        <p className="picker-notice">Synthetic demo crew. Not a medical device.</p>
        {crew.error && <p className="error">Could not open local storage: {crew.error}</p>}

        <h2 className="picker-group">Crew</h2>
        <div className="picker-list">
          {astronauts.map((m) => (
            <ProfileButton key={m.id} member={m} onSelect={onSelect} />
          ))}
        </div>
        <h2 className="picker-group">Medical team</h2>
        <div className="picker-list">
          {specialists.map((m) => (
            <ProfileButton key={m.id} member={m} onSelect={onSelect} />
          ))}
        </div>
      </div>
    </div>
  );
}

function ProfileButton({ member, onSelect }: { member: CrewMember; onSelect: (m: CrewMember) => void }) {
  return (
    <button type="button" className="profile-button" onClick={() => onSelect(member)}>
      <span className="avatar" aria-hidden="true">
        {member.display_name.replace("Dr. ", "").charAt(0)}
      </span>
      <span className="profile-text">
        <span className="profile-name">{member.display_name}</span>
        <span className="profile-position">{member.position}</span>
      </span>
    </button>
  );
}
