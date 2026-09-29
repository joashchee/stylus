import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";
import { ActivityStatus } from "./components/ActivityStatus";
import { AppTesting } from "./components/AppTesting";
import { ArtViewer, type Zoom } from "./components/ArtViewer";
import { Dialog } from "./components/Dialog";
import { AppMarkIcon, ChecklistIcon, FolderIcon, GearIcon, InfoIcon } from "./components/icons";
import { SaucePanel } from "./components/SaucePanel";
import { StartupScreen } from "./components/StartupScreen";
import { useActivities } from "./lib/activity";
import { closeArt, coreInfo, openArt, openExtensions, setRenderSettings, type CoreInfo, type OpenedArt, type RenderSettings } from "./lib/backend";
import { applyTheme, loadTheme, type Theme } from "./lib/theme";

/**
 * Launch-time work, in the order StartupScreen names it. Only the core
 * exists so far; font atlases, recent files and the theme pack join as
 * they land (Diskette's docs/stylus-notes.md, "Loading screen").
 */
const STARTUP_STEPS = ["core"] as const;
type StartupStep = (typeof STARTUP_STEPS)[number];
const STARTUP_LABELS: Record<StartupStep, string> = {
  core: "Starting the engine…",
};

function App() {
  const [startupPending, setStartupPending] = useState<StartupStep[]>([...STARTUP_STEPS]);
  const [theme, setTheme] = useState<Theme>(loadTheme);
  const [gearOpen, setGearOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [appTestingOpen, setAppTestingOpen] = useState(false);
  const [core, setCore] = useState<CoreInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [extensions, setExtensions] = useState<string[]>([]);
  const [art, setArt] = useState<OpenedArt | null>(null);
  const [zoom, setZoom] = useState<Zoom>("fit");
  const [dropActive, setDropActive] = useState(false);
  const gearRef = useRef<HTMLDivElement>(null);
  const { activities, runActivity } = useActivities();

  const finishStep = (step: StartupStep) => setStartupPending((list) => list.filter((s) => s !== step));

  useEffect(() => applyTheme(theme), [theme]);

  useEffect(() => {
    Promise.all([coreInfo(), openExtensions()])
      .then(([info, exts]) => {
        setCore(info);
        setExtensions(exts);
      })
      .catch((e) => setError(`Couldn't start the engine: ${e}`))
      .finally(() => finishStep("core"));
  }, []);

  /** Opens a file for viewing, replacing the art on screen. Never changes the file. */
  const openPath = useCallback(
    async (path: string) => {
      setError(null);
      const name = path.split(/[\\/]/).pop() ?? path;
      try {
        const opened = await runActivity(`Opening ${name}…`, () => openArt(path));
        setArt((previous) => {
          if (previous) void closeArt(previous.id);
          return opened;
        });
      } catch (e) {
        setError(String(e));
      }
    },
    [runActivity],
  );

  async function pickAndOpen() {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "ANSI and ASCII art", extensions: extensions.flatMap((e) => [e, e.toUpperCase()]) }],
    });
    if (typeof path === "string") await openPath(path);
  }

  async function changeSettings(patch: Partial<RenderSettings>) {
    if (!art) return;
    try {
      const info = await setRenderSettings(art.id, { ...art.info.settings, ...patch });
      setArt((current) => (current && current.id === art.id ? { ...current, info } : current));
    } catch (e) {
      setError(String(e));
    }
  }

  // Dropping a file on the window opens it (tauri.conf.json's dragDropEnabled).
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === "enter" || p.type === "over") setDropActive(true);
      else if (p.type === "leave") setDropActive(false);
      else if (p.type === "drop") {
        setDropActive(false);
        const path = p.paths.find((f) => extensions.includes(f.split(".").pop()?.toLowerCase() ?? ""));
        if (path) void openPath(path);
        else if (p.paths.length > 0) setError(`Stylus can't open that file. It opens ${extensions.map((e) => `.${e.toUpperCase()}`).join(", ")}.`);
      }
    });
    return () => void unlisten.then((f) => f());
  }, [extensions, openPath]);

  useEffect(() => {
    if (!gearOpen) return;
    const onDown = (e: MouseEvent) => {
      if (gearRef.current && !gearRef.current.contains(e.target as Node)) setGearOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setGearOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [gearOpen]);

  return (
    <div className="app">
      <StartupScreen
        progress={1 - startupPending.length / STARTUP_STEPS.length}
        label={startupPending.length > 0 ? STARTUP_LABELS[startupPending[0]] : ""}
        done={startupPending.length === 0}
      />
      <header className="top">
        <div className="title-group">
          <h1>
            <span className="app-mark">
              <AppMarkIcon />
            </span>
            <span>
              Stylus<span className="accent">.</span>
            </span>
            <span className="version-tag">v{__APP_VERSION__}</span>
          </h1>
          <p>ANSI art and animation, free and open source.</p>
        </div>
        <div className="header-actions">
          <div className="gear-menu-wrap" ref={gearRef}>
            <button
              type="button"
              className="small icontext-btn gear-btn"
              data-testid="gear-button"
              title="Settings & about"
              aria-haspopup="true"
              aria-expanded={gearOpen}
              onClick={() => setGearOpen((v) => !v)}
            >
              <GearIcon width={16} height={16} />
            </button>
            <div className={`gear-menu${gearOpen ? " open" : ""}`}>
              <label className="menu-item menu-item-checkbox">
                <input
                  type="checkbox"
                  data-testid="ansiapps-theme-toggle"
                  checked={theme === "ansiapps"}
                  onChange={(e) => setTheme(e.currentTarget.checked ? "ansiapps" : "modern")}
                />
                <span>ANSIapps theme (old-school DOS look)</span>
              </label>
              <div className="menu-sep" />
              <button
                type="button"
                className="menu-item"
                onClick={() => {
                  setGearOpen(false);
                  setAboutOpen(true);
                }}
              >
                <InfoIcon />
                <span>About Stylus</span>
              </button>
              {import.meta.env.DEV && (
                <>
                  <div className="menu-sep" />
                  <button
                    type="button"
                    className="menu-item"
                    onClick={() => {
                      setGearOpen(false);
                      setAppTestingOpen(true);
                    }}
                  >
                    <ChecklistIcon />
                    <span>App Testing</span>
                  </button>
                </>
              )}
            </div>
          </div>
        </div>
      </header>

      {error && <p className="error">{error}</p>}
      <ActivityStatus activities={activities} />

      {art ? (
        <div className="viewer-layout">
          <section className="panel viewer-panel">
            <h2 data-testid="art-name">{art.name}</h2>
            <div className="viewer-toolbar">
              <button type="button" className="small icontext-btn" onClick={() => void pickAndOpen()}>
                <span className="btn-icon">
                  <FolderIcon />
                </span>
                Open…
              </button>
              <label className="toggle" title="9-px character cells, like VGA text mode">
                <input
                  type="checkbox"
                  data-testid="toggle-letter-spacing"
                  checked={art.info.settings.letterSpacing}
                  onChange={(e) => void changeSettings({ letterSpacing: e.currentTarget.checked })}
                />
                <span>9-px spacing</span>
              </label>
              <label className="toggle" title="Bright backgrounds instead of blinking text">
                <input
                  type="checkbox"
                  data-testid="toggle-ice"
                  checked={art.info.settings.iceColors}
                  onChange={(e) => void changeSettings({ iceColors: e.currentTarget.checked })}
                />
                <span>iCE colors</span>
              </label>
              <label className="toggle" title="Stretch to the original screen's shape">
                <input
                  type="checkbox"
                  data-testid="toggle-aspect"
                  checked={art.info.settings.aspectRatio}
                  onChange={(e) => void changeSettings({ aspectRatio: e.currentTarget.checked })}
                />
                <span>Aspect ratio</span>
              </label>
              <label className="toggle">
                <span>Zoom</span>
                <select
                  data-testid="zoom-select"
                  value={String(zoom)}
                  onChange={(e) => setZoom(e.currentTarget.value === "fit" ? "fit" : (Number(e.currentTarget.value) as Zoom))}
                >
                  <option value="fit">Fit width</option>
                  <option value="1">100%</option>
                  <option value="2">200%</option>
                  <option value="3">300%</option>
                </select>
              </label>
            </div>
            <ArtViewer id={art.id} name={art.name} info={art.info} zoom={zoom} runActivity={runActivity} onError={setError} />
          </section>
          <SaucePanel name={art.name} info={art.info} />
        </div>
      ) : (
        <section className="panel empty-state">
          <h2>Open some art</h2>
          <p className="desc">
            Drop a file here or choose Open. Stylus opens {extensions.length > 0 ? extensions.map((e) => `.${e.toUpperCase()}`).join(", ") : "ANSI and ASCII art"}, and
            never changes the file.
          </p>
          <button type="button" className="primary icontext-btn" data-testid="open-button" onClick={() => void pickAndOpen()}>
            <span className="btn-icon">
              <FolderIcon />
            </span>
            Open…
          </button>
          <p className="muted mono core-line" data-testid="core-version">
            {core ? `stylus-core ${core.version} on icy_tools ${core.icyToolsRev.slice(0, 7)}` : "Engine not started"}
          </p>
        </section>
      )}

      {dropActive && (
        <div className="drop-overlay" aria-hidden="true">
          <div className="drop-box">Drop to open</div>
        </div>
      )}

      <Dialog open={aboutOpen} onClose={() => setAboutOpen(false)} title="About Stylus">
        <p>
          Stylus {__APP_VERSION__} opens, draws, converts and animates ANSI art, and is where the ANSIapps theme's graphics are made. It's free and open source under
          the MIT license, and everything happens on this computer.
        </p>
        <h3 className="about-section-title">Credits</h3>
        <p className="about-section-desc" data-testid="about-credits">
          Built on icy_tools by Mike Krüger (github.com/mkrueger/icy_tools), used under the MIT license.
        </p>
        <p className="about-section-desc">
          The ANSIapps theme's font is IBM VGA 8x16 from The Ultimate Oldschool PC Font Pack by VileR (int10h.org/oldschool-pc-fonts), licensed under CC BY-SA 4.0 and
          included unmodified.
        </p>
      </Dialog>

      {import.meta.env.DEV && <AppTesting open={appTestingOpen} onClose={() => setAppTestingOpen(false)} />}
    </div>
  );
}

export default App;
