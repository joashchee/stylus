import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";
import { ActivityStatus } from "./components/ActivityStatus";
import { AppTesting } from "./components/AppTesting";
import { ArtEditor } from "./components/ArtEditor";
import { Dialog } from "./components/Dialog";
import { SaveDialog, sauceFieldsOf } from "./components/SaveDialog";
import { ImageToAnsi } from "./components/ImageToAnsi";
import { MenuBar, ShortcutList } from "./components/MenuBar";
import { AppMarkIcon, ChecklistIcon, FolderIcon, GearIcon, InfoIcon } from "./components/icons";
import { StartupScreen } from "./components/StartupScreen";
import { useActivities } from "./lib/activity";
import {
  closeArt,
  converterLicense,
  coreInfo,
  imageExtensions,
  libraryNotices,
  listConverters,
  newArt,
  openArt,
  openExtensions,
  saveArt,
  saveFormats,
  saveLosses,
  setRenderSettings,
  setTextFont,
  textFonts,
  type ConverterInfo,
  type DocumentInfo,
  type SaveFormat,
  type LibraryNotice,
  type CoreInfo,
  type OpenedArt,
  type RenderSettings,
} from "./lib/backend";
import { applyTheme, loadTheme, type Theme } from "./lib/theme";
import { useCommands, type Handlers } from "./lib/commands";

/**
 * Launch-time work, in the order StartupScreen names it. Only the core
 * exists so far; font atlases, recent files and the theme pack join as
 * they land (Diskette's docs/stylus-notes.md, "Loading screen").
 */
const STARTUP_STEPS = ["core"] as const;

/** Canvas sizes New offers first (columns × rows). */
const NEW_SIZES: [number, number][] = [
  [80, 25],
  [80, 50],
  [132, 25],
  [160, 25],
];

/** The art on screen, and the file it came from (null until first saved). */
interface OpenDocument extends OpenedArt {
  path: string | null;
  /** The user agreed to Save replacing `path`, so later saves don't ask. */
  replaceConfirmed: boolean;
}
type StartupStep = (typeof STARTUP_STEPS)[number];
const STARTUP_LABELS: Record<StartupStep, string> = {
  core: "Starting the engine…",
};

function App() {
  const [startupPending, setStartupPending] = useState<StartupStep[]>([...STARTUP_STEPS]);
  const [theme, setTheme] = useState<Theme>(loadTheme);
  const [gearOpen, setGearOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const [appTestingOpen, setAppTestingOpen] = useState(false);
  const [core, setCore] = useState<CoreInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [extensions, setExtensions] = useState<string[]>([]);
  const [imageExts, setImageExts] = useState<string[]>([]);
  const [converters, setConverters] = useState<ConverterInfo[]>([]);
  const [libraries, setLibraries] = useState<LibraryNotice[]>([]);
  const [view, setView] = useState<"art" | "image">("art");
  const [droppedImage, setDroppedImage] = useState<{ path: string; at: number } | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [licenseShown, setLicenseShown] = useState<{ id: string; text: string } | null>(null);
  const [art, setArt] = useState<OpenDocument | null>(null);
  const [formats, setFormats] = useState<SaveFormat[]>([]);
  const [fonts, setFonts] = useState<string[]>([]);
  const [saveMode, setSaveMode] = useState<"save" | "save-as" | null>(null);
  const [newOpen, setNewOpen] = useState(false);
  const [newSize, setNewSize] = useState({ columns: 80, rows: 25, ice: true });
  /** Something waiting on "discard the unsaved changes?" */
  const [discardThen, setDiscardThen] = useState<(() => void) | null>(null);
  const artRef = useRef<OpenDocument | null>(null);
  artRef.current = art;
  const [dropActive, setDropActive] = useState(false);
  const gearRef = useRef<HTMLDivElement>(null);
  const { activities, runActivity } = useActivities();

  const finishStep = (step: StartupStep) => setStartupPending((list) => list.filter((s) => s !== step));

  useEffect(() => applyTheme(theme), [theme]);

  useEffect(() => {
    Promise.all([coreInfo(), openExtensions(), imageExtensions(), listConverters(), libraryNotices(), saveFormats(), textFonts()])
      .then(([info, exts, imgExts, convs, libs, saveFmts, txtFonts]) => {
        setFormats(saveFmts);
        setFonts(txtFonts);
        setLibraries(libs);
        setCore(info);
        setExtensions(exts);
        setImageExts(imgExts);
        setConverters(convs);
      })
      .catch((e) => setError(`Couldn't start the engine: ${e}`))
      .finally(() => finishStep("core"));
  }, []);

  /** Runs `then` now, or after the user agrees to drop unsaved changes. */
  const unlessEdited = useCallback((then: () => void) => {
    if (artRef.current?.info.edited) setDiscardThen(() => then);
    else then();
  }, []);

  const replaceArt = useCallback((next: OpenDocument) => {
    setArt((previous) => {
      if (previous) void closeArt(previous.id);
      return next;
    });
    setView("art");
  }, []);

  /** Opens a file, replacing the art on screen. Never changes the file. */
  const openPath = useCallback(
    (path: string) =>
      unlessEdited(async () => {
        setError(null);
        const name = path.split(/[\\/]/).pop() ?? path;
        try {
          const opened = await runActivity(`Opening ${name}…`, () => openArt(path));
          replaceArt({ ...opened, path, replaceConfirmed: false });
        } catch (e) {
          setError(String(e));
        }
      }),
    [runActivity, unlessEdited, replaceArt],
  );

  async function createNew() {
    setNewOpen(false);
    try {
      const made = await newArt(newSize.columns, newSize.rows, newSize.ice);
      replaceArt({ ...made, path: null, replaceConfirmed: false });
    } catch (e) {
      setError(String(e));
    }
  }

  const updateInfo = useCallback((info: DocumentInfo) => {
    setArt((current) => (current ? { ...current, info } : current));
  }, []);

  /** Save: straight to the document's own file once the user has agreed to replace it and nothing would be lost. */
  async function saveCurrent() {
    const doc = artRef.current;
    if (!doc) return;
    const ext = doc.path?.split(".").pop()?.toLowerCase() ?? "";
    if (!doc.path || !formats.some((f) => f.extension === ext)) {
      setSaveMode("save-as");
      return;
    }
    if (doc.replaceConfirmed) {
      try {
        const losses = await saveLosses(doc.id, ext);
        if (losses.length === 0) {
          const path = doc.path;
          const info = await runActivity(`Saving ${doc.name}…`, () => saveArt(doc.id, path, sauceFieldsOf(doc.info), true));
          onSaved(info, path);
          return;
        }
      } catch (e) {
        setError(`Couldn't save: ${e}`);
        return;
      }
    }
    setSaveMode("save");
  }

  // The menu bar's File, View and Help commands, and the render settings
  // (lib/commands.ts). The editor adds its own while the Art workspace shows.
  const onArt = view === "art" && !!art;
  const handlers: Handlers = {
    "file.new": { run: () => setNewOpen(true) },
    "file.open": { run: () => void pickAndOpen() },
    "file.save": { run: () => void saveCurrent(), enabled: onArt },
    "file.saveAs": { run: () => setSaveMode("save-as"), enabled: onArt },
    "colors.ice": { run: () => void changeSettings({ iceColors: !art?.info.settings.iceColors }), enabled: onArt, checked: !!art?.info.settings.iceColors },
    "view.letterSpacing": { run: () => void changeSettings({ letterSpacing: !art?.info.settings.letterSpacing }), enabled: onArt, checked: !!art?.info.settings.letterSpacing },
    "view.aspect": { run: () => void changeSettings({ aspectRatio: !art?.info.settings.aspectRatio }), enabled: onArt, checked: !!art?.info.settings.aspectRatio },
    "help.shortcuts": { run: () => setShortcutsOpen(true) },
    "help.about": { run: () => setAboutOpen(true) },
  };
  // A text file without SAUCE can be shown in the Amiga fonts.
  if (onArt && art.info.format === "ASCII" && !art.info.sauce) {
    const current = fonts.find((f) => art.info.font.includes(f.replace("Amiga ", ""))) ?? "IBM VGA";
    for (const font of fonts) {
      handlers[`view.textFont.${font}`] = { run: () => void changeTextFont(font), checked: font === current, label: `Font: ${font}` };
    }
  }
  useCommands("app", handlers);

  // The window's title names the art, with • for unsaved changes.
  const title = art ? `${art.name}${art.info.edited ? " •" : ""} — Stylus` : "Stylus";
  useEffect(() => {
    document.title = title;
    void getCurrentWindow()
      .setTitle(title)
      .catch(() => undefined);
  }, [title]);

  function onSaved(info: DocumentInfo, path: string) {
    const name = path.split(/[\\/]/).pop() ?? path;
    setArt((current) => (current ? { ...current, info, path, name, replaceConfirmed: true } : current));
    setSaveMode(null);
    setStatus(`Saved ${name}`);
  }

  async function changeTextFont(font: string) {
    if (!art) return;
    try {
      updateInfo(await setTextFont(art.id, font));
      try {
        localStorage.setItem(`stylus.textFont.${art.name.split(".").pop()?.toLowerCase()}`, font);
      } catch {
        // Remembering is a convenience; the font still changed.
      }
    } catch (e) {
      setError(String(e));
    }
  }

  // A text file without SAUCE opens in the font last chosen for its kind (Amiga ASCII).
  useEffect(() => {
    if (!art || art.info.sauce || art.info.format !== "ASCII") return;
    let font: string | null = null;
    try {
      font = localStorage.getItem(`stylus.textFont.${art.name.split(".").pop()?.toLowerCase()}`);
    } catch {
      font = null;
    }
    if (font && font !== art.info.font && fonts.includes(font)) void changeTextFont(font);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [art?.id, fonts]);

  // Closing the window with unsaved changes asks first.
  useEffect(() => {
    const unlisten = getCurrentWindow().onCloseRequested((event) => {
      if (!artRef.current?.info.edited) return;
      event.preventDefault();
      setDiscardThen(() => () => void getCurrentWindow().destroy());
    });
    return () => void unlisten.then((f) => f());
  }, []);

  async function pickAndOpen() {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "ANSI and ASCII art", extensions: extensions.flatMap((e) => [e, e.toUpperCase()]) }],
    });
    if (typeof path === "string") openPath(path);
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

  // Dropping a file on the window opens it (tauri.conf.json's dragDropEnabled):
  // art in the viewer, an image in Image to ANSI.
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === "enter" || p.type === "over") setDropActive(true);
      else if (p.type === "leave") setDropActive(false);
      else if (p.type === "drop") {
        setDropActive(false);
        const ext = (f: string) => f.split(".").pop()?.toLowerCase() ?? "";
        const artPath = p.paths.find((f) => extensions.includes(ext(f)));
        const imagePath = p.paths.find((f) => imageExts.includes(ext(f)));
        if (artPath) {
          openPath(artPath);
        } else if (imagePath) {
          setView("image");
          setDroppedImage({ path: imagePath, at: Date.now() });
        } else if (p.paths.length > 0) {
          setError(
            `Stylus can't open that file. It opens ${extensions.map((e) => `.${e.toUpperCase()}`).join(", ")}, and converts images (${imageExts
              .map((e) => `.${e.toUpperCase()}`)
              .join(", ")}) to ANSI.`,
          );
        }
      }
    });
    return () => void unlisten.then((f) => f());
  }, [extensions, imageExts, openPath]);

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
      <MenuBar
        start={
          <span className="menu-app" aria-label={`Stylus ${__APP_VERSION__}`}>
            <span className="app-mark">
              <AppMarkIcon />
            </span>
            <span>
              Stylus<span className="accent">.</span>
            </span>
          </span>
        }
        end={
          <>
            <nav className="workspaces" aria-label="Workspaces">
              <button
                type="button"
                className={`view-tab${view === "art" ? " selected" : ""}`}
                aria-pressed={view === "art"}
                data-testid="tab-art"
                onClick={() => setView("art")}
              >
                Art
              </button>
              <button
                type="button"
                className={`view-tab${view === "image" ? " selected" : ""}`}
                aria-pressed={view === "image"}
                data-testid="tab-image-to-ansi"
                title="Make art from an image"
                onClick={() => setView("image")}
              >
                Make
              </button>
            </nav>
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
          </>
        }
      />

      {error && <p className="error">{error}</p>}
      {status && !error && (
        <p className="status-message" role="status">
          {status}
        </p>
      )}
      <ActivityStatus activities={activities} />

      <div hidden={view !== "image"}>
        <ImageToAnsi
          converters={converters}
          extensions={imageExts}
          droppedPath={droppedImage}
          runActivity={runActivity}
          onError={setError}
          onStatus={setStatus}
        />
      </div>

      <div hidden={view !== "art"}>
        {art ? (
          <ArtEditor
            art={art}
            onInfo={updateInfo}
            runActivity={runActivity}
            onError={setError}
            active={view === "art" && saveMode === null && !newOpen && discardThen === null}
          />
        ) : (
          <section className="panel empty-state">
            <h2>Open some art, or start new</h2>
            <p className="desc">
              Drop a file here or choose Open. Stylus opens {extensions.length > 0 ? extensions.map((e) => `.${e.toUpperCase()}`).join(", ") : "ANSI and ASCII art"}, and
              never changes the file unless you save over it.
            </p>
            <div className="empty-actions">
              <button type="button" className="primary" data-testid="new-empty-button" onClick={() => setNewOpen(true)}>
                New…
              </button>
              <button type="button" className="primary icontext-btn" data-testid="open-button" onClick={() => void pickAndOpen()}>
                <span className="btn-icon">
                  <FolderIcon />
                </span>
                Open…
              </button>
            </div>
            <p className="muted mono core-line" data-testid="core-version">
              {core ? `stylus-core ${core.version} on icy_tools ${core.icyToolsRev.slice(0, 7)}` : "Engine not started"}
            </p>
          </section>
        )}
      </div>

      {art && saveMode && (
        <SaveDialog
          open
          art={art}
          path={saveMode === "save" ? art.path : null}
          formats={formats}
          onClose={() => setSaveMode(null)}
          onSaved={onSaved}
          onError={setError}
        />
      )}

      <Dialog
        open={newOpen}
        onClose={() => setNewOpen(false)}
        title="New art"
        actions={
          <>
            <button type="button" onClick={() => setNewOpen(false)}>
              Cancel
            </button>
            <button type="button" className="primary" data-testid="new-confirm" onClick={() => unlessEdited(() => void createNew())}>
              Create
            </button>
          </>
        }
      >
        <div className="size-presets">
          {NEW_SIZES.map(([c, r]) => (
            <button
              key={`${c}x${r}`}
              type="button"
              className={`small${newSize.columns === c && newSize.rows === r ? " selected" : ""}`}
              aria-pressed={newSize.columns === c && newSize.rows === r}
              onClick={() => setNewSize({ ...newSize, columns: c, rows: r })}
            >
              {c}×{r}
            </button>
          ))}
        </div>
        <label className="form-row">
          <span>Columns</span>
          <input
            type="number"
            min={1}
            max={2000}
            data-testid="new-columns"
            value={newSize.columns}
            onChange={(e) => setNewSize({ ...newSize, columns: Math.max(1, Math.min(2000, Number(e.currentTarget.value) || 1)) })}
          />
        </label>
        <label className="form-row">
          <span>Rows</span>
          <input
            type="number"
            min={1}
            max={100000}
            data-testid="new-rows"
            value={newSize.rows}
            onChange={(e) => setNewSize({ ...newSize, rows: Math.max(1, Math.min(100000, Number(e.currentTarget.value) || 1)) })}
          />
        </label>
        <label className="toggle">
          <input type="checkbox" checked={newSize.ice} onChange={(e) => setNewSize({ ...newSize, ice: e.currentTarget.checked })} />
          <span>iCE colors (16 backgrounds, no blink)</span>
        </label>
        <p>Typing past the last row adds rows as you go.</p>
      </Dialog>

      <Dialog
        open={discardThen !== null}
        onClose={() => setDiscardThen(null)}
        title="Unsaved changes"
        actions={
          <>
            <button type="button" onClick={() => setDiscardThen(null)}>
              Keep editing
            </button>
            <button
              type="button"
              className="danger danger-fill"
              data-testid="discard-confirm"
              onClick={() => {
                const then = discardThen;
                setDiscardThen(null);
                then?.();
              }}
            >
              Discard changes
            </button>
          </>
        }
      >
        <p>{art?.name} has changes that aren't saved. Discard them?</p>
      </Dialog>

      {dropActive && (
        <div className="drop-overlay" aria-hidden="true">
          <div className="drop-box">Drop to open art or convert an image</div>
        </div>
      )}

      <Dialog open={shortcutsOpen} onClose={() => setShortcutsOpen(false)} title="Keyboard shortcuts" className="dialog-wide">
        <ShortcutList />
      </Dialog>

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
        <h3 className="about-section-title">Image to ANSI converters</h3>
        <p className="about-section-desc">
          Image to ANSI carries Rust ports of these open-source converters, each under its own license. Choose a license to read it in full.
        </p>
        <ul className="about-converters" data-testid="about-converters">
          {converters.map((c) => (
            <li key={c.id}>
              <span>
                {c.name} ({c.origin}). {c.copyright}.{" "}
              </span>
              <button
                type="button"
                className="link-btn"
                onClick={() =>
                  licenseShown?.id === c.id
                    ? setLicenseShown(null)
                    : void converterLicense(c.id).then((text) => setLicenseShown({ id: c.id, text }), (e) => setError(String(e)))
                }
              >
                {c.license} license
              </button>
              {licenseShown?.id === c.id && <pre className="license-text">{licenseShown.text}</pre>}
            </li>
          ))}
        </ul>
        <p className="about-section-desc">The ports also carry code ported from these libraries:</p>
        <ul className="about-converters" data-testid="about-libraries">
          {libraries.map((l) => (
            <li key={l.name}>
              <details>
                <summary>
                  {l.name} ({l.origin}), {l.license} license: {l.usedFor}
                </summary>
                <pre className="license-text">{l.licenseText}</pre>
              </details>
            </li>
          ))}
        </ul>
      </Dialog>

      {import.meta.env.DEV && <AppTesting open={appTestingOpen} onClose={() => setAppTestingOpen(false)} />}
    </div>
  );
}

export default App;
