import { useEffect, useState } from "react";
import { getSyncStatus, type CrewMember } from "../api";
import { useCommand } from "../useCommand";
import brandMark from "../assets/brand-mark.svg";
import { navGroupsFor, type Screen } from "../navigation";
import { Icon } from "./Icon";

type Props = {
  current: Screen;
  onNavigate: (screen: Screen) => void;
  member: CrewMember;
  onSwitchUser: () => void;
  version: string | null;
};

export function Sidebar({ current, onNavigate, member, onSwitchUser, version }: Props) {
  // Re-read on navigation and every 10 s so the indicator follows background sync.
  const [tick, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 10_000);
    return () => clearInterval(id);
  }, []);
  const sync = useCommand(getSyncStatus, [current, tick]);
  const last = sync.data?.last;
  const configured = sync.data?.server_configured ?? false;
  const connectionText = !configured
    ? "Local mode · no server configured"
    : last?.outcome === "failed"
      ? `Offline · ${sync.data?.pending ?? 0} waiting to send`
      : `Server linked · ${sync.data?.pending ?? 0} waiting`;
  const connectionClass = !configured ? "dot-local" : last?.outcome === "failed" ? "dot-offline" : "dot-online";
  return (
    <aside className="sidebar">
      <div className="brand">
        <img src={brandMark} alt="" width={28} height={28} />
        <span>Nurbit</span>
      </div>

      <nav aria-label="Main navigation" className="nav">
        {navGroupsFor(member.role).map((group) => (
          <div key={group.title} className="nav-group">
            <div className="nav-group-title">{group.title}</div>
            {group.items.map((item) => (
              <button
                key={item.screen}
                type="button"
                className="nav-item"
                aria-current={item.screen === current ? "page" : undefined}
                onClick={() => onNavigate(item.screen)}
              >
                <Icon name={item.icon} />
                <span>{item.label}</span>
              </button>
            ))}
          </div>
        ))}
      </nav>

      <div className="sidebar-footer">
        <div className="user">
          <span className="avatar avatar-small" aria-hidden="true">
            {member.display_name.replace("Dr. ", "").charAt(0)}
          </span>
          <span className="user-text">
            <span className="user-name">{member.display_name}</span>
            <span className="user-position">{member.position}</span>
          </span>
          <button type="button" className="switch-user" onClick={onSwitchUser}>
            Switch
          </button>
        </div>
        <div className="connection">
          <span className={`dot ${connectionClass}`} aria-hidden="true" />
          {connectionText}
        </div>
        <div className="version">{version ? `Version ${version} (beta)` : ""}</div>
      </div>
    </aside>
  );
}
