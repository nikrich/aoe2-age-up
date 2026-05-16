// Open Age — embedded interactive overlay demo.
// Renders the real overlay UI (View switcher → Overlay/Library/Settings)
// using the same DOM structure as the source repo.

function StepCard({ step, variant }) {
  const v = step.villagers_assigned || {};
  const hasV = (v.food || 0) + (v.wood || 0) + (v.gold || 0) + (v.stone || 0) > 0;
  return (
    <div className={`step-card step-card--${variant}`}>
      <div className="step-action">{step.action}</div>
      {step.notes && <div className="step-notes">{step.notes}</div>}
      {hasV && (
        <div className="step-villagers">
          {v.food > 0 && <span className="resource food">F:{v.food}</span>}
          {v.wood > 0 && <span className="resource wood">W:{v.wood}</span>}
          {v.gold > 0 && <span className="resource gold">G:{v.gold}</span>}
          {v.stone > 0 && <span className="resource stone">S:{v.stone}</span>}
        </div>
      )}
    </div>
  );
}

function OverlayView({ buildOrder, currentStep, capturing, gameState, onPrev, onReset, onNext, onToggleCapture }) {
  const total = buildOrder.steps.length;
  const cur = buildOrder.steps[currentStep];
  const nxt = currentStep < total - 1 ? buildOrder.steps[currentStep + 1] : null;
  return (
    <div className="overlay">
      <div className="overlay-header">
        <span className="overlay-title">{buildOrder.name}</span>
        <span className="step-counter">{currentStep + 1} / {total}</span>
      </div>
      {cur && <StepCard step={cur} variant="current" />}
      {nxt && <StepCard step={nxt} variant="next" />}
      {gameState && (
        <div className="game-state-bar">
          <span className="resource food">F:{gameState.food}</span>
          <span className="resource wood">W:{gameState.wood}</span>
          <span className="resource gold">G:{gameState.gold}</span>
          <span className="resource stone">S:{gameState.stone}</span>
          <span className="resource vils">V:{gameState.villagers}</span>
          <span className="resource time">
            {Math.floor(gameState.game_time_seconds / 60)}:{String(gameState.game_time_seconds % 60).padStart(2, "0")}
          </span>
        </div>
      )}
      <div style={{ flex: 1, minHeight: 8 }} />
      <div className="overlay-nav">
        <button className="nav-btn" onClick={onPrev}>Prev</button>
        <button className="nav-btn" onClick={onReset}>Reset</button>
        <button className="nav-btn" onClick={onNext}>Next</button>
        <button className={`nav-btn ${capturing ? "nav-btn--active" : ""}`} onClick={onToggleCapture}>
          {capturing ? "Stop" : "Capture"}
        </button>
      </div>
    </div>
  );
}

function LibraryView({ buildOrder, onPick }) {
  return (
    <div className="library">
      <div className="library-header">Build orders</div>
      {window.BUILD_ORDERS.map((bo) => (
        <button
          key={bo.id}
          className={`library-item ${buildOrder.id === bo.id ? "library-item--active" : ""}`}
          onClick={() => onPick(bo)}
        >
          <div className="library-item-name">{bo.name}</div>
          <div className="library-item-civ">{bo.civilization} · {bo.difficulty}</div>
          <div className="library-item-desc">{bo.description}</div>
          <div className="library-item-tags">
            {bo.tags.map((t) => <span key={t} className="tag">{t}</span>)}
          </div>
        </button>
      ))}
    </div>
  );
}

function SettingsView({ capturing }) {
  const k = window.HOTKEYS;
  return (
    <div className="settings">
      <div className="settings-header">Settings</div>
      <div className="settings-row"><span className="settings-label">Capture</span><span className="settings-value">{capturing ? "Active · 1 Hz" : "Paused"}</span></div>
      <div className="settings-row"><span className="settings-label">OCR backend</span><span className="settings-value">Template</span></div>
      <div className="settings-row"><span className="settings-label">Auto-advance</span><span className="settings-value">on</span></div>
      <div className="settings-row"><span className="settings-label">Click-through</span><span className="settings-value">off</span></div>
      <div className="settings-row"><span className="settings-label">Opacity</span><span className="settings-value">0.90</span></div>
      <div className="settings-row" style={{borderTop: "1px solid var(--border)", marginTop: 8, paddingTop: 12}}><span className="settings-label">Advance</span><span className="settings-value">{k.advance_step}</span></div>
      <div className="settings-row"><span className="settings-label">Previous</span><span className="settings-value">{k.previous_step}</span></div>
      <div className="settings-row"><span className="settings-label">Reset</span><span className="settings-value">{k.reset}</span></div>
      <div className="settings-row"><span className="settings-label">Pause capture</span><span className="settings-value">{k.pause_capture}</span></div>
      <div className="settings-row"><span className="settings-label">Toggle overlay</span><span className="settings-value">{k.toggle_visibility}</span></div>
      <div className="settings-row"><span className="settings-label">Click-through</span><span className="settings-value">{k.toggle_click_through}</span></div>
    </div>
  );
}

function OverlayDemo({ initialBuildOrderId, autoplay, autoplayMs = 2200, scale = 1, showClose = true }) {
  const [buildOrder, setBuildOrder] = React.useState(() =>
    window.BUILD_ORDERS.find((b) => b.id === (initialBuildOrderId || "scouts-generic"))
  );
  const [view, setView] = React.useState("overlay"); // overlay / library / settings
  const [currentStep, setCurrentStep] = React.useState(0);
  const [capturing, setCapturing] = React.useState(true);
  const [gameState, setGameState] = React.useState({
    food: 120, wood: 80, gold: 0, stone: 0, villagers: 18, game_time_seconds: 252
  });

  // Live game-state ticker — simulates OCR output drifting over time.
  React.useEffect(() => {
    if (!capturing) { setGameState(null); return; }
    setGameState((s) => s || { food: 120, wood: 80, gold: 0, stone: 0, villagers: 18, game_time_seconds: 252 });
    let t = 0;
    const id = setInterval(() => {
      t += 1;
      setGameState((s) => s && {
        ...s,
        food: Math.max(0, s.food + Math.round((Math.random() - 0.4) * 6)),
        wood: Math.max(0, s.wood + Math.round((Math.random() - 0.3) * 4)),
        gold: Math.max(0, s.gold + Math.round((Math.random() - 0.6) * 3)),
        stone: s.stone,
        villagers: s.villagers + (t % 8 === 0 ? 1 : 0),
        game_time_seconds: s.game_time_seconds + 1
      });
    }, 1000);
    return () => clearInterval(id);
  }, [capturing]);

  // Autoplay — advance through steps
  React.useEffect(() => {
    if (!autoplay || view !== "overlay") return;
    const id = setInterval(() => {
      setCurrentStep((i) => (i + 1) % buildOrder.steps.length);
    }, autoplayMs);
    return () => clearInterval(id);
  }, [autoplay, autoplayMs, view, buildOrder]);

  const advance = () => setCurrentStep((i) => Math.min(buildOrder.steps.length - 1, i + 1));
  const previous = () => setCurrentStep((i) => Math.max(0, i - 1));
  const reset = () => setCurrentStep(0);

  return (
    <div className="app" style={{ transform: scale !== 1 ? `scale(${scale})` : undefined, transformOrigin: "center" }}>
      <div className="view-switcher" data-tauri-drag-region>
        <button className={`view-btn ${view === "overlay" ? "view-btn--active" : ""}`} onClick={() => setView("overlay")}>Overlay</button>
        <button className={`view-btn ${view === "library" ? "view-btn--active" : ""}`} onClick={() => setView("library")}>Library</button>
        <button className={`view-btn ${view === "settings" ? "view-btn--active" : ""}`} onClick={() => setView("settings")}>Settings</button>
        {showClose && <button className="close-btn" aria-label="Close">✕</button>}
      </div>
      {view === "overlay" && (
        <OverlayView
          buildOrder={buildOrder}
          currentStep={currentStep}
          capturing={capturing}
          gameState={gameState}
          onPrev={previous}
          onReset={reset}
          onNext={advance}
          onToggleCapture={() => setCapturing((c) => !c)}
        />
      )}
      {view === "library" && (
        <LibraryView
          buildOrder={buildOrder}
          onPick={(bo) => { setBuildOrder(bo); setCurrentStep(0); setView("overlay"); }}
        />
      )}
      {view === "settings" && <SettingsView capturing={capturing} />}
    </div>
  );
}

Object.assign(window, { OverlayDemo, OverlayView, LibraryView, SettingsView, StepCard });
