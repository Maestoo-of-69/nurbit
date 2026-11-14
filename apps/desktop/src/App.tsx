// App shell: profile picker first, then sidebar + the open screen.
// `refreshKey` goes up after each save so screens reload their data.
import { useState } from "react";
import { getAppInfo, type CrewMember } from "./api";
import { Sidebar } from "./components/Sidebar";
import { homeScreenFor, type Screen } from "./navigation";
import { CrewScreen } from "./screens/specialist/CrewScreen";
import { PendingReviewsScreen } from "./screens/specialist/PendingReviewsScreen";
import { AboutScreen } from "./screens/AboutScreen";
import { DailyCheckScreen } from "./screens/DailyCheckScreen";
import { HistoryScreen } from "./screens/HistoryScreen";
import { OverviewScreen } from "./screens/OverviewScreen";
import { AssistantScreen } from "./screens/AssistantScreen";
import { SyncScreen } from "./screens/PlannedScreens";
import { ProfilePicker } from "./screens/ProfilePicker";
import { RisksScreen } from "./screens/RisksScreen";
import { useCommand } from "./useCommand";

const FALLBACK_NOTICE = "Prototype - synthetic data only - not a medical device";

export default function App() {
  const [member, setMember] = useState<CrewMember | null>(null);
  const [screen, setScreen] = useState<Screen>("overview");
  const [refreshKey, setRefreshKey] = useState(0);
  const info = useCommand(getAppInfo);

  if (!member) {
    return (
      <ProfilePicker
        onSelect={(m) => {
          setMember(m);
          setScreen(homeScreenFor(m.role));
        }}
      />
    );
  }

  const refresh = () => setRefreshKey((k) => k + 1);

  return (
    <div className="shell">
      <Sidebar
        current={screen}
        onNavigate={setScreen}
        member={member}
        onSwitchUser={() => setMember(null)}
        version={info.data?.version ?? null}
      />
      <main className="content">
        <div className="notice" role="note">
          {info.data?.notice ?? FALLBACK_NOTICE}
        </div>
        {info.error && <p className="error">Could not reach the application core: {info.error}</p>}
        <div className="screen" key={`${screen}-${member.id}`}>
          {renderScreen(screen, setScreen, member, refreshKey, refresh, info.data)}
        </div>
      </main>
    </div>
  );
}

function renderScreen(
  screen: Screen,
  navigate: (screen: Screen) => void,
  member: CrewMember,
  refreshKey: number,
  refresh: () => void,
  info: Parameters<typeof AboutScreen>[0]["info"],
) {
  switch (screen) {
    case "crew":
      return <CrewScreen actor={member} refreshKey={refreshKey} onChanged={refresh} />;
    case "pending-reviews":
      return <PendingReviewsScreen actor={member} refreshKey={refreshKey} onChanged={refresh} />;
    case "overview":
      return <OverviewScreen member={member} refreshKey={refreshKey} onNavigate={navigate} />;
    case "daily-check":
      return <DailyCheckScreen member={member} onSaved={refresh} />;
    case "history":
      return <HistoryScreen member={member} refreshKey={refreshKey} />;
    case "risks":
      return <RisksScreen member={member} refreshKey={refreshKey} />;
    case "assistant":
      return <AssistantScreen member={member} />;
    case "sync":
      return <SyncScreen refreshKey={refreshKey} />;
    case "about":
      return <AboutScreen info={info} />;
  }
}
