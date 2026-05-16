// Age Up — landing page sections.
// Imports OverlayDemo and renders the full marketing layout.

const SOCIAL_ICON = (id) => (
  <svg viewBox="0 0 24 24" aria-hidden="true">
    <use href={`assets/icons.svg#${id}`} />
  </svg>
);

function Logo({ size = 24 }) {
  return (
    <span className="logo">
      <img src="landing/assets/logo.svg" alt="" width={size} height={size} />
      <span>Age Up</span>
    </span>
  );
}

function Header() {
  return (
    <header className="site-header">
      <div className="site-header__inner">
        <Logo />
        <nav className="nav">
          <a href="#how-it-works">How it works</a>
          <a href="#build-orders">Build orders</a>
          <a href="#performance">Performance</a>
          <a href="#roadmap">Roadmap</a>
        </nav>
        <div className="nav-spacer"></div>
        <div className="nav-actions">
          <a href="#" className="gh-link" aria-label="GitHub repository">
            <svg viewBox="0 0 24 24" aria-hidden="true"><use href="landing/assets/icons.svg#github-icon" /></svg>
            <span>nikrich/open-age</span>
            <span className="stars">★ 1.2k</span>
          </a>
          <a href="#download" className="btn btn--primary" style={{padding: "8px 14px", fontSize: 13}}>Download</a>
        </div>
      </div>
    </header>
  );
}

function Hero() {
  return (
    <section className="hero container">
      <div className="hero__grid">
        <div className="hero__copy">
          <span className="hero__badge">
            <span className="pip">v0.4 · alpha</span>
            <span>Phase 1 shipped — manual-advance overlay live</span>
          </span>
          <h1 className="hero__title">
            Run your build order <span className="accent">without leaving the game.</span>
          </h1>
          <p className="hero__lede">
            Age Up is a transparent, always-on-top desktop overlay for <strong>Age of Empires II: Definitive Edition</strong>. It walks you through any YAML build order step-by-step — and reads the game with OCR, so you don't have to.
          </p>
          <div className="hero__cta">
            <a href="#download" className="btn btn--primary">
              Download for Windows
              <span className="kbd">v0.4</span>
            </a>
            <a href="#" className="btn btn--ghost">
              <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true"><use href="landing/assets/icons.svg#github-icon" /></svg>
              View source
            </a>
          </div>
          <div className="hero__meta">
            <span><span className="dot"></span>Free · MIT · Open source</span>
            <span>Tauri 2.0 · 12 MB</span>
            <span>No memory injection</span>
          </div>
        </div>

        <div className="hero__stage">
          <div className="hero__overlay-mount">
            <OverlayDemo initialBuildOrderId="scouts-generic" autoplay={t => t.heroAutoplay} />
          </div>
        </div>
      </div>
    </section>
  );
}

function Proof() {
  return (
    <section className="proof">
      <div className="container proof__inner">
        <span className="proof__label">// Performance budget — measured on a 2021 ThinkPad, AoE2:DE 1080p</span>
        <div className="proof__stats">
          <div className="proof__stat">
            <div className="v">&lt; 2<span style={{fontSize: 18, opacity: 0.7}}>%</span></div>
            <div className="l">CPU at idle</div>
          </div>
          <div className="proof__stat">
            <div className="v">90<span style={{fontSize: 18, opacity: 0.7}}>ms</span></div>
            <div className="l">OCR cycle, p95</div>
          </div>
          <div className="proof__stat">
            <div className="v">320<span style={{fontSize: 18, opacity: 0.7}}>×480</span></div>
            <div className="l">overlay footprint</div>
          </div>
          <div className="proof__stat">
            <div className="v">0</div>
            <div className="l">memory reads</div>
          </div>
        </div>
      </div>
    </section>
  );
}

function Features() {
  const items = [
    {
      title: "Always on top, always 0.85α",
      desc: "A 320×480 glass panel pinned over the game. Click-through when you need it; full focus when you don't.",
      icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><rect x="3" y="3" width="14" height="14" rx="1.5"/><rect x="7" y="7" width="14" height="14" rx="1.5"/></svg>
    },
    {
      title: "OCR, not memory injection",
      desc: "Age Up reads the resource bar by screen-capture + Tesseract. No DLL hooks, no detection vectors, no TOS risk.",
      icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><circle cx="11" cy="11" r="6.5"/><path d="m20 20-4.3-4.3"/></svg>
    },
    {
      title: "YAML build orders",
      desc: "Steps are plain text — author your own, share them as files, version them with git. Three samples ship in the box.",
      icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><path d="M14 3v6h6M8 13h8M8 17h6"/></svg>
    },
    {
      title: "Auto-advance on triggers",
      desc: "Step 4 fires when villagers ≥ 11. Step 6 fires at 5:30 game time. The overlay watches; you keep clicking.",
      icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg>
    },
    {
      title: "Hotkeys for everything",
      desc: "Prev / Next / Reset / Pause capture / Toggle overlay — all bindable. The overlay never steals focus from the game.",
      icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><rect x="3" y="6" width="18" height="12" rx="2"/><path d="M7 10h.01M11 10h.01M15 10h.01M7 14h10"/></svg>
    },
    {
      title: "Made for AoE2 vocabulary",
      desc: "Vills, TC, lure, flush, click up. F W G S V. The overlay speaks the language you already use at 7 APM.",
      icon: <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><path d="M4 7h16M4 12h16M4 17h10"/></svg>
    }
  ];
  return (
    <section className="container section section--tight" id="features">
      <span className="eyebrow">Features</span>
      <h2 className="section-title">A coach standing <em>behind your shoulder.</em></h2>
      <p className="section-sub">
        Six things the overlay does well. Each is a deliberate constraint — Age Up is a single-purpose tool, not a launcher, not a tracker, not a coach AI.
      </p>
      <div className="features">
        {items.map((it, i) => (
          <div className="feature" key={i}>
            <div className="feature__icon">{it.icon}</div>
            <h3 className="feature__title">{it.title}</h3>
            <p className="feature__desc">{it.desc}</p>
          </div>
        ))}
      </div>
    </section>
  );
}

function HowItWorks() {
  return (
    <section className="container section" id="how-it-works">
      <span className="eyebrow">Pipeline</span>
      <h2 className="section-title">From your screen to <em>the next step.</em></h2>
      <p className="section-sub">
        Every second, Age Up screenshots the AoE2 window, crops to four pre-calibrated rectangles, runs OCR on each, and tests your build order's triggers. If a trigger matches, the current step advances. Otherwise, you do nothing and the panel stays put.
      </p>
      <div className="pipeline">
        <div className="stage">
          <div className="stage__num">01 · CAPTURE</div>
          <div className="stage__name">Screen-grab the game</div>
          <div className="stage__desc">scrap crate captures the AoE2 window at 1 Hz. Region rectangles are stored from the calibration wizard.</div>
          <div className="stage__art">
            <div><span className="k">window</span> <span className="arrow">→</span> <span className="v">"Age of Empires II"</span></div>
            <div><span className="k">crop</span> <span className="arrow">→</span> <span className="v">[food, wood, gold, stone]</span></div>
            <div><span className="k">interval</span> <span className="arrow">→</span> <span className="v">1000ms</span></div>
          </div>
          <div className="stage__connector">→</div>
        </div>
        <div className="stage">
          <div className="stage__num">02 · OCR</div>
          <div className="stage__name">Read the resource bar</div>
          <div className="stage__desc">Tesseract digit-only mode parses each rectangle. Template-matching fallback handles AoE2's stylized digit glyphs.</div>
          <div className="stage__art">
            <div><span className="k">food</span> <span className="arrow">→</span> <span className="v">120</span></div>
            <div><span className="k">wood</span> <span className="arrow">→</span> <span className="v">80</span></div>
            <div><span className="k">vils</span> <span className="arrow">→</span> <span className="v">18</span></div>
          </div>
          <div className="stage__connector">→</div>
        </div>
        <div className="stage">
          <div className="stage__num">03 · MATCH</div>
          <div className="stage__name">Test step triggers</div>
          <div className="stage__desc">Each step's trigger expression runs against the parsed state. Triggers are simple: <code style={{fontSize: 11}}>villagers &gt;= 11</code>, <code style={{fontSize: 11}}>game_time &gt;= 330</code>.</div>
          <div className="stage__art">
            <div><span className="k">step.4</span> <span className="arrow">→</span> <span className="v">vils &gt;= 11</span> <span className="arrow">→</span> <span className="a">match</span></div>
            <div><span className="k">step.5</span> <span className="arrow">→</span> <span className="v">food &gt;= 200</span> <span className="arrow">→</span> wait</div>
          </div>
          <div className="stage__connector">→</div>
        </div>
        <div className="stage">
          <div className="stage__num">04 · ADVANCE</div>
          <div className="stage__name">Update the overlay</div>
          <div className="stage__desc">React state ticks forward. The amber left rule jumps to the new step. No animation, no fanfare — you keep playing.</div>
          <div className="stage__art">
            <div><span className="k">current</span> <span className="arrow">→</span> <span className="a">step.4</span></div>
            <div><span className="k">next</span> <span className="arrow">→</span> <span className="v">step.5</span></div>
            <div><span className="k">render</span> <span className="arrow">→</span> <span className="v">&lt; 16ms</span></div>
          </div>
        </div>
      </div>
    </section>
  );
}

function BuildOrders() {
  return (
    <section className="container section" id="build-orders">
      <span className="eyebrow">Build orders</span>
      <h2 className="section-title">Three in the box. <em>Yours next.</em></h2>
      <p className="section-sub">
        Age Up ships with a Scout Rush, a Britons Archer flush, and a Fast Castle. They live as plain YAML files in <code style={{fontFamily: "var(--font-mono)", color: "var(--violet-bright)"}}>~/Documents/AgeUp/build-orders/</code>. Drop your own beside them.
      </p>
      <div className="bo-grid">
        {window.BUILD_ORDERS.map((bo) => (
          <div className="bo-card" key={bo.id}>
            <div className="bo-card__head">
              <div className="bo-card__name">{bo.name}</div>
              <div className="bo-card__civ">{bo.civilization}</div>
            </div>
            <div className="bo-card__desc">{bo.description}</div>
            <div className="bo-card__tags">
              {bo.tags.map((t) => <span key={t} className="tag">{t}</span>)}
            </div>
            <div className="bo-card__steps">
              <span>{bo.difficulty}</span>
              <span><span className="accent">{bo.steps.length}</span> steps</span>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}

function Performance() {
  const [cpu, setCpu] = React.useState(1.4);
  const [ocr, setOcr] = React.useState(82);
  React.useEffect(() => {
    const id = setInterval(() => {
      setCpu(0.8 + Math.random() * 1.6);
      setOcr(70 + Math.round(Math.random() * 30));
    }, 1400);
    return () => clearInterval(id);
  }, []);
  return (
    <section className="container section" id="performance">
      <span className="eyebrow">Performance & privacy</span>
      <h2 className="section-title">Nothing the game cares about. <em>Nothing you didn't ask for.</em></h2>
      <p className="section-sub">
        Age Up has no business reading your memory, your network traffic, or your account. Here's exactly what it does and doesn't do.
      </p>
      <div className="specs">
        <div className="spec-list">
          <div className="spec">
            <div className="spec__k">Capture</div>
            <div className="spec__v">Cross-platform <code style={{fontFamily: "var(--font-mono)"}}>scrap</code> crate at 1 Hz. <small>~30 ms per frame on integrated graphics.</small></div>
          </div>
          <div className="spec">
            <div className="spec__k">OCR</div>
            <div className="spec__v">Tesseract 5 in digit-only mode, with a template-matching fallback for AoE2's stylized glyphs. <small>p95 cycle: 90 ms.</small></div>
          </div>
          <div className="spec">
            <div className="spec__k">Memory access</div>
            <div className="spec__v">None. The overlay never reads or writes the game's process memory. <small>Anti-cheat compliant by construction.</small></div>
          </div>
          <div className="spec">
            <div className="spec__k">Network</div>
            <div className="spec__v">None at runtime. Build orders are loaded from disk; no telemetry, no auto-update phone-home. <small>Updater opt-in, signed.</small></div>
          </div>
          <div className="spec">
            <div className="spec__k">Footprint</div>
            <div className="spec__v">12 MB installer. ~80 MB RAM. <small>Tauri 2.0 — webview, no Electron.</small></div>
          </div>
          <div className="spec">
            <div className="spec__k">Hotkeys</div>
            <div className="spec__v">Six global bindings. The overlay never steals focus from the game window. <small>Click-through toggle for hands-off play.</small></div>
          </div>
        </div>

        <div className="specs__panel">
          <div className="h">
            <span>// runtime · live</span>
            <span className="live">capturing</span>
          </div>
          <div className="row"><span className="k">cpu</span><span className="v ok">{cpu.toFixed(1)} %</span></div>
          <div className="bar"><div className="fill" style={{width: `${Math.min(100, cpu * 30)}%`}}></div></div>
          <div className="row" style={{marginTop: 16}}><span className="k">ocr.cycle</span><span className="v ok">{ocr} ms</span></div>
          <div className="bar"><div className="fill" style={{width: `${(ocr / 200) * 100}%`}}></div></div>
          <div className="row" style={{marginTop: 16}}><span className="k">ram</span><span className="v">82 MB</span></div>
          <div className="row" style={{marginTop: 4}}><span className="k">window</span><span className="v">320×480</span></div>
          <div className="row" style={{marginTop: 4}}><span className="k">opacity</span><span className="v">0.85</span></div>
          <div className="row" style={{marginTop: 4}}><span className="k">always_on_top</span><span className="v ok">true</span></div>
          <div className="row" style={{marginTop: 4}}><span className="k">click_through</span><span className="v warn">false</span></div>
          <div style={{marginTop: 16, paddingTop: 12, borderTop: "1px solid var(--border)", color: "var(--text-tertiary)", fontSize: 11}}>
            % open-age · idle · 1.4% cpu · 82 MB
          </div>
        </div>
      </div>
    </section>
  );
}

function Roadmap() {
  const phases = [
    { num: "P1", name: "Manual overlay", desc: "Always-on-top window, library, settings, hotkeys.", status: "done", state: "Shipped" },
    { num: "P2", name: "Calibration", desc: "Drag rectangles to mark resource regions on a screenshot.", status: "current", state: "In progress" },
    { num: "P3", name: "Live capture", desc: "scrap @ 1 Hz, region cropping, ImageBuf pipeline.", status: "next", state: "Next up" },
    { num: "P4", name: "OCR + auto-advance", desc: "Tesseract integration, trigger evaluation, auto step.", status: "next", state: "Q3" },
    { num: "P5", name: "Editor + polish", desc: "Form-based YAML authoring, more BOs, civ presets.", status: "next", state: "Q4" }
  ];
  return (
    <section className="container section" id="roadmap">
      <span className="eyebrow">Roadmap</span>
      <h2 className="section-title">Five phases. <em>One you can use today.</em></h2>
      <p className="section-sub">
        Age Up is built in public. Phase 1 (the panel you're holding above) is live; phases 2–5 ship as they're ready. Track progress on GitHub.
      </p>
      <div className="roadmap">
        {phases.map((p) => (
          <div className={`phase phase--${p.status}`} key={p.num}>
            <div className="phase__dot"></div>
            <div className="phase__num">{p.num}</div>
            <div className="phase__name">{p.name}</div>
            <div className="phase__desc">{p.desc}</div>
            <div className="phase__status">{p.state}</div>
          </div>
        ))}
      </div>
    </section>
  );
}

function Voice() {
  const rules = [
    { bad: "Please make sure to send 4 villagers to the woodline now!", good: "4 vills to wood (build lumber camp)" },
    { bad: "We were unable to load your build order, sorry about that.", good: "Build order failed to load: invalid trigger on step 4." },
    { bad: "Click here to advance to the next age 🎉", good: "Click up to Feudal Age" },
    { bad: "Hooray, you reached the Castle Age!", good: "Castle Age reached at 17:42" }
  ];
  return (
    <section className="container section" id="voice">
      <span className="eyebrow">Voice</span>
      <h2 className="section-title">Built for players competing for <em>their own attention.</em></h2>
      <p className="section-sub">
        The overlay is competing with a real-time strategy game for every pixel and every word. Imperative, second-person, no fluff. Resources are letters. Numbers are tabular. Emoji are forbidden.
      </p>
      <div className="voice">
        <div className="voice-rules">
          {rules.map((r, i) => (
            <div className="rule" key={i}>
              <div className="rule__bad">{r.bad}</div>
              <div className="rule__good">{r.good}</div>
            </div>
          ))}
        </div>
        <div className="voice-quote">
          "Build mill on berries, 3 vills to berries.<br/>
          Lure second boar.<br/>
          2 more vills to wood.<br/>
          Click up to Feudal Age."
          <div className="meta">— scouts-generic.yaml · steps 5–9</div>
        </div>
      </div>
    </section>
  );
}

function OpenSourceCTA() {
  return (
    <section className="container" id="download">
      <div className="os-cta">
        <div>
          <h2 className="os-cta__title">Open source. <span style={{color: "var(--violet-bright)"}}>MIT.</span> Yours to fork.</h2>
          <p className="os-cta__sub">
            Age Up is built with Tauri 2.0, React 19, and a thin Rust core. The build order schema is plain YAML. Issues, PRs, and BO contributions welcome.
          </p>
          <div className="os-cta__actions">
            <a href="#" className="btn btn--primary">
              Download v0.4 · Windows
            </a>
            <a href="#" className="btn btn--ghost">
              <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true"><use href="landing/assets/icons.svg#github-icon" /></svg>
              github.com/nikrich/open-age
            </a>
          </div>
        </div>
        <div className="os-cta__terminal">
          <div className="os-cta__terminal-bar">
            <span className="dot"></span><span className="dot"></span><span className="dot"></span>
            <span className="title">~ / open-age</span>
          </div>
          <div className="os-cta__terminal-body">
            <div><span className="prompt">$</span><span className="cmd">git clone github.com/nikrich/open-age</span></div>
            <div className="out">cloning into 'open-age'... <span className="ok">done.</span></div>
            <div><span className="prompt">$</span><span className="cmd">cd open-age && pnpm tauri dev</span></div>
            <div className="out">▸ vite v6.0.0  ready in 412 ms</div>
            <div className="out">▸ rust  building open-age v0.4.0</div>
            <div className="out">▸ overlay <span className="ok">window opened</span> · pid 4212</div>
            <div><span className="prompt">$</span><span style={{color: "var(--accent)"}}>_</span></div>
          </div>
        </div>
      </div>
    </section>
  );
}

function Footer() {
  return (
    <footer>
      <div className="container">
        <div className="footer__inner">
          <div className="footer__brand">
            <Logo />
            <p className="footer__tagline">
              An always-on-top desktop overlay for AoE2:DE that walks you through any build order. Open source, OCR-driven, no memory injection.
            </p>
          </div>
          <div className="footer__cols">
            <div className="footer__col">
              <h4>Product</h4>
              <ul>
                <li><a href="#download">Download</a></li>
                <li><a href="#features">Features</a></li>
                <li><a href="#performance">Performance</a></li>
                <li><a href="#roadmap">Roadmap</a></li>
              </ul>
            </div>
            <div className="footer__col">
              <h4>Build orders</h4>
              <ul>
                <li><a href="#">Scouts (Generic)</a></li>
                <li><a href="#">Archers (Britons)</a></li>
                <li><a href="#">Fast Castle Knights</a></li>
                <li><a href="#">Author your own</a></li>
              </ul>
            </div>
            <div className="footer__col">
              <h4>Open source</h4>
              <ul>
                <li><a href="#">GitHub</a></li>
                <li><a href="#">spec.md</a></li>
                <li><a href="#">CLAUDE.md</a></li>
                <li><a href="#">Contributing</a></li>
              </ul>
            </div>
          </div>
        </div>
        <div className="footer__bottom">
          <span>// open-age · v0.4 · MIT · 2026</span>
          <div className="footer__social">
            <a href="#" aria-label="GitHub"><svg viewBox="0 0 24 24"><use href="landing/assets/icons.svg#github-icon" /></svg></a>
            <a href="#" aria-label="X"><svg viewBox="0 0 24 24"><use href="landing/assets/icons.svg#x-icon" /></svg></a>
            <a href="#" aria-label="Bluesky"><svg viewBox="0 0 24 24"><use href="landing/assets/icons.svg#bluesky-icon" /></svg></a>
            <a href="#" aria-label="Discord"><svg viewBox="0 0 24 24"><use href="landing/assets/icons.svg#discord-icon" /></svg></a>
          </div>
        </div>
      </div>
    </footer>
  );
}

function HeroWithVariant({ tweaks }) {
  return (
    <section className="hero container">
      <div className="hero__grid">
        <div className="hero__copy">
          <span className="hero__badge">
            <span className="pip">v0.4 · alpha</span>
            <span>Phase 1 shipped — manual-advance overlay live</span>
          </span>
          <h1 className="hero__title">
            Run your build order <span className="accent">without leaving the game.</span>
          </h1>
          <p className="hero__lede">
            Age Up is a transparent, always-on-top desktop overlay for <strong>Age of Empires II: Definitive Edition</strong>. It walks you through any YAML build order step-by-step — and reads the game with OCR, so you don't have to.
          </p>
          <div className="hero__cta">
            <a href="#download" className="btn btn--primary">
              Download for Windows
              <span className="kbd">v0.4</span>
            </a>
            <a href="#" className="btn btn--ghost">
              <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true"><use href="landing/assets/icons.svg#github-icon" /></svg>
              View source
            </a>
          </div>
          <div className="hero__meta">
            <span><span className="dot"></span>Free · MIT · Open source</span>
            <span>Tauri 2.0 · 12 MB</span>
            <span>No memory injection</span>
          </div>
        </div>
        <div className="hero__stage">
          <div className="hero__overlay-mount">
            <OverlayDemo
              key={tweaks.heroVariant}
              initialBuildOrderId={tweaks.heroBO || "scouts-generic"}
              autoplay={tweaks.heroVariant === "animated"}
              autoplayMs={2400}
            />
          </div>
        </div>
      </div>
    </section>
  );
}

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "palette": "amber",
  "bg": "default",
  "density": "default",
  "heroVariant": "interactive",
  "heroBO": "scouts-generic",
  "showRoadmap": true
}/*EDITMODE-END*/;

function Page() {
  const [tweaks, setTweak] = useTweaks(TWEAK_DEFAULTS);

  React.useEffect(() => {
    const root = document.documentElement;
    root.dataset.palette = tweaks.palette;
    root.dataset.bg = tweaks.bg;
    root.dataset.density = tweaks.density;
  }, [tweaks.palette, tweaks.bg, tweaks.density]);

  return (
    <>
      <Header />
      <main>
        <HeroWithVariant tweaks={tweaks} />
        <Proof />
        <Features />
        <HowItWorks />
        <BuildOrders />
        <Performance />
        <Voice />
        {tweaks.showRoadmap && <Roadmap />}
        <OpenSourceCTA />
      </main>
      <Footer />

      <TweaksPanel title="Tweaks">
        <TweakSection label="Palette">
          <TweakRadio
            label="Identity"
            value={tweaks.palette}
            onChange={(v) => setTweak("palette", v)}
            options={[
              { value: "mix", label: "Mix" },
              { value: "violet", label: "Violet" },
              { value: "amber", label: "Amber" }
            ]}
          />
        </TweakSection>
        <TweakSection label="Hero demo">
          <TweakRadio
            label="Behaviour"
            value={tweaks.heroVariant}
            onChange={(v) => setTweak("heroVariant", v)}
            options={[
              { value: "interactive", label: "Interactive" },
              { value: "animated", label: "Animated" }
            ]}
          />
          <TweakSelect
            label="Build order"
            value={tweaks.heroBO}
            onChange={(v) => setTweak("heroBO", v)}
            options={window.BUILD_ORDERS.map(b => ({ value: b.id, label: b.name }))}
          />
        </TweakSection>
        <TweakSection label="Layout">
          <TweakRadio
            label="Background"
            value={tweaks.bg}
            onChange={(v) => setTweak("bg", v)}
            options={[{ value: "default", label: "Dark" }, { value: "darker", label: "Darker" }]}
          />
          <TweakRadio
            label="Density"
            value={tweaks.density}
            onChange={(v) => setTweak("density", v)}
            options={[
              { value: "cozy", label: "Cozy" },
              { value: "default", label: "Standard" },
              { value: "roomy", label: "Roomy" }
            ]}
          />
          <TweakToggle
            label="Show roadmap"
            value={tweaks.showRoadmap}
            onChange={(v) => setTweak("showRoadmap", v)}
          />
        </TweakSection>
      </TweaksPanel>
    </>
  );
}

window.Page = Page;
