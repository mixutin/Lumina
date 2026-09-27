import { useCallback, useEffect, useMemo, useState } from "react";
import {
  defaultStatus,
  getSystemStatus,
  launchHoyoplay,
  prepareLumina,
  type RuntimeStatus,
} from "./lib/bridge";

type Tone = "good" | "warn" | "muted";

function LogoMark() {
  return (
    <div className="logo-mark" aria-hidden="true">
      <span className="logo-orbit logo-orbit-a" />
      <span className="logo-orbit logo-orbit-b" />
      <span className="logo-core" />
    </div>
  );
}

function Icon({ name }: { name: "home" | "runtime" | "logs" | "settings" | "play" | "spark" | "refresh" }) {
  const paths = {
    home: <><path d="M3 10.5 12 3l9 7.5"/><path d="M5.5 9.5V21h13V9.5"/><path d="M9 21v-7h6v7"/></>,
    runtime: <><rect x="3" y="4" width="18" height="16" rx="3"/><path d="M7 9h10M7 13h4M7 17h7"/></>,
    logs: <><path d="M4 5h16v14H4z"/><path d="M7 9h6M7 13h10M7 17h7"/></>,
    settings: <><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06-2.86 2.86-.06-.06A1.7 1.7 0 0 0 15 19.4a1.7 1.7 0 0 0-1 .6 1.7 1.7 0 0 0-.4 1V21H9.6v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.88.34l-.06.06-2.86-2.86.06-.06A1.7 1.7 0 0 0 4.1 15a1.7 1.7 0 0 0-.6-1 1.7 1.7 0 0 0-1-.4H2.4V9.6h.1A1.7 1.7 0 0 0 4.1 8.5a1.7 1.7 0 0 0-.34-1.88l-.06-.06L6.56 3.7l.06.06A1.7 1.7 0 0 0 8.5 4.1a1.7 1.7 0 0 0 1-.6 1.7 1.7 0 0 0 .4-1v-.1h4.04v.1a1.7 1.7 0 0 0 1.06 1.6 1.7 1.7 0 0 0 1.88-.34l.06-.06 2.86 2.86-.06.06A1.7 1.7 0 0 0 19.4 8.5a1.7 1.7 0 0 0 .6 1 1.7 1.7 0 0 0 1 .4h.1v4.04H21A1.7 1.7 0 0 0 19.4 15Z"/></>,
    play: <path d="m9 7 8 5-8 5V7Z"/>,
    spark: <path d="m12 2 1.2 5.1L18 8.5l-4.8 1.4L12 15l-1.2-5.1L6 8.5l4.8-1.4L12 2Zm6 12 .7 2.8 2.3.7-2.3.7L18 21l-.7-2.8-2.3-.7 2.3-.7L18 14Z"/>,
    refresh: <><path d="M20 6v5h-5"/><path d="M4.6 9A8 8 0 0 1 18 6l2 5M4 18v-5h5"/><path d="M19.4 15A8 8 0 0 1 6 18l-2-5"/></>,
  };

  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      {paths[name]}
    </svg>
  );
}

function StatusPill({ label, tone }: { label: string; tone: Tone }) {
  return <span className={`status-pill ${tone}`}><span className="status-dot" />{label}</span>;
}

function HealthCard({
  eyebrow,
  title,
  detail,
  tone,
}: {
  eyebrow: string;
  title: string;
  detail: string;
  tone: Tone;
}) {
  return (
    <article className="health-card glass">
      <div>
        <p className="eyebrow">{eyebrow}</p>
        <h3>{title}</h3>
      </div>
      <StatusPill label={detail} tone={tone} />
    </article>
  );
}

export default function App() {
  const [status, setStatus] = useState<RuntimeStatus>(defaultStatus);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<"prepare" | "launch" | null>(null);
  const [message, setMessage] = useState("Checking your Linux environment…");

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const next = await getSystemStatus();
      setStatus(next);
      setMessage("System scan complete.");
    } catch {
      setMessage("Desktop bridge is unavailable in browser preview. Run Lumina through Tauri for live diagnostics.");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const readiness = useMemo(() => {
    const checks = [status.umuAvailable, status.vulkanAvailable, status.hoyoplayInstalled];
    return Math.round((checks.filter(Boolean).length / checks.length) * 100);
  }, [status]);

  async function prepare() {
    setBusy("prepare");
    try {
      const result = await prepareLumina();
      setMessage(result);
      await refresh();
    } catch (error) {
      setMessage(`Preparation failed: ${String(error)}`);
    } finally {
      setBusy(null);
    }
  }

  async function launch() {
    setBusy("launch");
    try {
      setMessage(await launchHoyoplay());
    } catch (error) {
      setMessage(`Launch failed: ${String(error)}`);
    } finally {
      setBusy(null);
    }
  }

  return (
    <main className="app-shell">
      <div className="ambient ambient-one" />
      <div className="ambient ambient-two" />

      <aside className="sidebar">
        <div className="brand">
          <LogoMark />
          <div>
            <strong>LUMINA</strong>
            <span>Linux launcher</span>
          </div>
        </div>

        <nav className="nav-list" aria-label="Primary">
          <button className="nav-item active"><Icon name="home" />Overview</button>
          <button className="nav-item"><Icon name="runtime" />Runtime</button>
          <button className="nav-item"><Icon name="logs" />Logs</button>
          <button className="nav-item"><Icon name="settings" />Settings</button>
        </nav>

        <div className="sidebar-foot">
          <span className="version-chip">PRE-ALPHA · 0.1</span>
          <p>Official launcher.<br />Linux-native experience.</p>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">SYSTEM / OVERVIEW</p>
            <h1>Good evening, Proxy.</h1>
          </div>
          <button className="icon-button" onClick={() => void refresh()} aria-label="Refresh diagnostics">
            <Icon name="refresh" />
          </button>
        </header>

        <section className="hero glass">
          <div className="hero-grid" />
          <div className="hero-copy">
            <div className="hero-kicker"><Icon name="spark" /> READY FOR NEW ERIDU</div>
            <h2>Zenless.<br /><span>Without Steam.</span></h2>
            <p>
              Lumina wraps the official HoYoPlay launcher in a managed Linux runtime,
              keeping updates official and compatibility understandable.
            </p>
            <div className="hero-actions">
              <button className="primary-button" onClick={() => void launch()} disabled={!status.hoyoplayInstalled || busy !== null}>
                <Icon name="play" /> {busy === "launch" ? "Launching…" : "Launch HoYoPlay"}
              </button>
              <button className="secondary-button" onClick={() => void prepare()} disabled={busy !== null}>
                {busy === "prepare" ? "Preparing…" : "Prepare Lumina"}
              </button>
            </div>
          </div>
          <div className="hero-orb" aria-hidden="true">
            <div className="orb-ring ring-a" />
            <div className="orb-ring ring-b" />
            <div className="orb-ring ring-c" />
            <div className="orb-center"><LogoMark /></div>
          </div>
        </section>

        <div className="content-grid">
          <section className="panel glass">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">RUNTIME HEALTH</p>
                <h2>Launch readiness</h2>
              </div>
              <div className="readiness">
                <strong>{loading ? "—" : `${readiness}%`}</strong>
                <span>ready</span>
              </div>
            </div>

            <div className="health-list">
              <HealthCard eyebrow="GRAPHICS" title="Vulkan" detail={status.vulkanAvailable ? "Detected" : "Needs check"} tone={status.vulkanAvailable ? "good" : "warn"} />
              <HealthCard eyebrow="RUNTIME" title="UMU" detail={status.umuAvailable ? "Available" : "Not found"} tone={status.umuAvailable ? "good" : "warn"} />
              <HealthCard eyebrow="LAUNCHER" title="HoYoPlay" detail={status.hoyoplayInstalled ? "Installed" : "Not installed"} tone={status.hoyoplayInstalled ? "good" : "muted"} />
            </div>
          </section>

          <section className="panel glass">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">ENVIRONMENT</p>
                <h2>Your system</h2>
              </div>
            </div>
            <dl className="facts">
              <div><dt>Platform</dt><dd>{status.platform}</dd></div>
              <div><dt>Architecture</dt><dd>{status.arch}</dd></div>
              <div><dt>Session</dt><dd>{status.session}</dd></div>
              <div><dt>Data directory</dt><dd className="mono">{status.dataDir}</dd></div>
            </dl>
          </section>
        </div>

        <section className="activity-bar">
          <span className={`activity-light ${loading ? "pulse" : ""}`} />
          <div>
            <strong>{loading ? "Scanning…" : "Lumina status"}</strong>
            <p>{message}</p>
          </div>
        </section>
      </section>
    </main>
  );
}
