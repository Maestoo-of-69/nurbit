// Sync screen: queue status, server settings, manual sync.
import { useState } from "react";
import { getSyncStatus, saveSyncSettings, syncNow, type SyncReport } from "../api";
import { ScreenHeader } from "../components/ScreenHeader";
import { useCommand } from "../useCommand";

function describe(report: SyncReport) {
  const time = new Date(report.at).toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  switch (report.outcome) {
    case "not_configured":
      return `${time} · server not configured`;
    case "nothing_to_send":
      return `${time} · nothing to send`;
    case "failed":
      return `${time} · ${report.error ?? "failed"}`;
    case "ok":
      return `${time} · sent ${report.accepted}, already on server ${report.duplicates}, conflicts ${report.conflicts}, rejected ${report.invalid}`;
  }
}

export function SyncScreen({ refreshKey }: { refreshKey: number }) {
  const [version, setVersion] = useState(0);
  const status = useCommand(getSyncStatus, [refreshKey, version]);
  const [url, setUrl] = useState<string | null>(null);
  const [token, setToken] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);
  const s = status.data;
  const shownUrl = url ?? s?.server_url ?? "http://127.0.0.1:8787";

  async function act(action: () => Promise<string>) {
    setIsBusy(true);
    setMessage(null);
    try {
      setMessage(await action());
    } catch (e: unknown) {
      setMessage(String(e));
    } finally {
      setIsBusy(false);
      setVersion((v) => v + 1);
    }
  }

  return (
    <>
      <ScreenHeader title="Sync" subtitle="Sending records to the medical team when a link is available." />
      {status.error && <p className="error">{status.error}</p>}
      <div className="summary-grid">
        <section className="panel">
          <div className="panel-label">Waiting to send</div>
          <div className="panel-value">{s?.pending ?? "—"}</div>
          <p className="panel-text">Saved on this computer, queued for the server.</p>
        </section>
        <section className="panel">
          <div className="panel-label">Sent</div>
          <div className="panel-value">{s?.sent ?? "—"}</div>
          <p className="panel-text">Confirmed by the server (new or already stored).</p>
        </section>
        <section className="panel">
          <div className="panel-label">Conflicts</div>
          <div className={`panel-value ${s?.conflict ? "" : "panel-value-quiet"}`}>{s?.conflict ?? "—"}</div>
          <p className="panel-text">Server holds a different version; both are kept for review.</p>
        </section>
      </div>

      <section className="panel panel-wide">
        <div className="panel-heading">
          <div className="panel-label">Server</div>
          <span className="cell-note">{s?.server_configured ? "Configured" : "Local mode · not configured"}</span>
        </div>
        <div className="button-row">
          <input className="text-input" aria-label="Server address" value={shownUrl} onChange={(e) => setUrl(e.target.value)} />
          <input
            className="text-input"
            type="password"
            aria-label="Access token"
            placeholder={s?.server_configured ? "Token saved · leave empty to keep" : "Access token"}
            value={token}
            onChange={(e) => setToken(e.target.value)}
          />
        </div>
        <div className="button-row">
          <button
            type="button"
            className="button-secondary"
            disabled={isBusy}
            onClick={() =>
              act(async () => {
                await saveSyncSettings(shownUrl, token);
                setToken("");
                return "Settings saved.";
              })
            }
          >
            Save settings
          </button>
          <button
            type="button"
            className="button-primary"
            disabled={isBusy || !s?.server_configured}
            onClick={() => act(async () => describe(await syncNow()))}
          >
            Sync now
          </button>
        </div>
        {message && <p className="panel-text sync-message">{message}</p>}
        {s?.last && <p className="cell-note">Last attempt: {describe(s.last)}</p>}
        <p className="cell-note">
          Automatic retry every 30 seconds. Plain HTTP is for a local demo only; real deployments need HTTPS.
          Beta: the token is stored unencrypted on this computer.
        </p>
      </section>
      <p className="muted-text">Queued records are never overwritten. Synthetic demo history is never sent.</p>
    </>
  );
}
