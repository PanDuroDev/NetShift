import {
  Activity,
  ArrowDownRight,
  ArrowRight,
  ArrowUpRight,
  Bell,
  Cable,
  Check,
  ChevronDown,
  ChevronRight,
  CircleAlert,
  CircleHelp,
  Clock3,
  Copy,
  FileText,
  Globe,
  LayoutDashboard,
  Laptop,
  LockKeyhole,
  Moon,
  MoreHorizontal,
  Network,
  RefreshCw,
  Route,
  Search,
  Server,
  Settings,
  ShieldCheck,
  SlidersHorizontal,
  Sun,
  X,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useRef, useState, type FormEvent, type ReactNode } from "react";

type ViewId = "overview" | "providers" | "diagnostics" | "settings";
type Theme = "dark" | "light";

type DialogContent = {
  title: string;
  message: string;
  detail?: string;
};

const sampleValues = {
  ipv4: "192.0.2.10",
  ipv6: "2001:db8::1",
  mac: "02:00:5e:10:00:00",
  port: "443",
  interfaceId: "Ethernet 2 / ifIndex 12",
  log: "2026-10-31T10:20:30Z INFO route 192.0.2.10 -> 192.0.2.1 via Ethernet 2",
  path: String.raw`C:\Program Files\NetShift\logs\netshift.log`,
  gateway: "192.0.2.1",
  dns: "192.0.2.53",
};

const navItems: Array<{ id: ViewId; label: string; icon: LucideIcon }> = [
  { id: "overview", label: "Overview", icon: LayoutDashboard },
  { id: "providers", label: "Providers", icon: Network },
  { id: "diagnostics", label: "Diagnostics", icon: Activity },
  { id: "settings", label: "Settings", icon: Settings },
];

const providerItems = [
  {
    name: "Cloudflare WARP",
    description: "Device-scoped encrypted traffic routing.",
    badge: "Not connected",
    icon: Globe,
    detail: "WARP is shown as a sample provider. The prototype does not detect, install, or control it.",
  },
  {
    name: "WireGuard",
    description: "A lightweight VPN tunnel interface.",
    badge: "Not configured",
    icon: ShieldCheck,
    detail: "WireGuard is shown as a sample provider. No tunnel configuration is loaded.",
  },
  {
    name: "OpenVPN",
    description: "A configurable VPN client connection.",
    badge: "Not configured",
    icon: LockKeyhole,
    detail: "OpenVPN is shown as a sample provider. No client process or service is queried.",
  },
];

const activityItems = [
  {
    title: "Sample route snapshot loaded",
    description: "Local demo data · no operating-system query",
    time: "Just now",
    icon: Route,
  },
  {
    title: "Provider catalog displayed",
    description: "Three illustrative providers · no provider discovery",
    time: "2 min ago",
    icon: Network,
  },
  {
    title: "Safety boundary verified",
    description: "No network-changing commands are exposed",
    time: "5 min ago",
    icon: ShieldCheck,
  },
];

function PageHeading({
  eyebrow,
  title,
  description,
  action,
}: {
  eyebrow: string;
  title: string;
  description: string;
  action?: ReactNode;
}) {
  return (
    <div className="page-heading">
      <div>
        <p className="eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="page-description">{description}</p>
      </div>
      {action ? <div className="page-heading-action">{action}</div> : null}
    </div>
  );
}

function StatusTag({
  children,
  tone = "neutral",
}: {
  children: ReactNode;
  tone?: "neutral" | "positive" | "warning";
}) {
  return <span className={`status-tag status-tag--${tone}`}>{children}</span>;
}

function CopyField({
  label,
  value,
  onCopy,
  compact = false,
}: {
  label: string;
  value: string;
  onCopy: (label: string, value: string) => void;
  compact?: boolean;
}) {
  const copy = () => onCopy(label, value);

  return (
    <div className={compact ? "copy-field copy-field--compact" : "copy-field"}>
      <code dir="ltr">{value}</code>
      <button
        className="icon-button copy-button"
        type="button"
        onClick={() => void copy()}
        aria-label={`Copy ${label}`}
        title={`Copy ${label}`}
      >
        <Copy size={15} aria-hidden="true" />
      </button>
    </div>
  );
}

function MetricCard({
  icon: Icon,
  label,
  value,
  detail,
  trend,
  onCopy,
  copyLabel,
}: {
  icon: LucideIcon;
  label: string;
  value: string;
  detail: string;
  trend?: "up" | "down";
  onCopy?: (label: string, value: string) => void;
  copyLabel?: string;
}) {
  return (
    <article className="surface-card metric-card">
      <div className="metric-card-top">
        <span className="metric-icon">
          <Icon size={18} aria-hidden="true" />
        </span>
        <span className="metric-label">{label}</span>
        {trend ? (
          <span className={`metric-trend metric-trend--${trend}`}>
            {trend === "up" ? <ArrowUpRight size={14} /> : <ArrowDownRight size={14} />}
          </span>
        ) : null}
      </div>
      <div className="metric-value-row">
        <strong className="metric-value" dir="ltr">{value}</strong>
        {onCopy && copyLabel ? (
          <button
            type="button"
            className="icon-button metric-copy"
            aria-label={`Copy ${copyLabel}`}
            onClick={() => onCopy(copyLabel, value)}
          >
            <Copy size={14} aria-hidden="true" />
          </button>
        ) : null}
      </div>
      <p className="metric-detail">{detail}</p>
    </article>
  );
}

function App() {
  const [view, setView] = useState<ViewId>("overview");
  const [theme, setTheme] = useState<Theme>("dark");
  const [query, setQuery] = useState("");
  const [announcement, setAnnouncement] = useState("");
  const [lastUpdated, setLastUpdated] = useState("Just now");
  const [dialog, setDialog] = useState<DialogContent | null>(null);
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.dir = "ltr";
    document.documentElement.lang = "en";
  }, [theme]);

  useEffect(() => {
    const element = dialogRef.current;
    if (dialog && element && !element.open) {
      element.showModal();
    } else if (!dialog && element?.open) {
      element.close();
    }
  }, [dialog]);

  const copyToClipboard = async (label: string, value: string) => {
    try {
      if (!navigator.clipboard?.writeText) throw new Error("Clipboard API is unavailable.");
      await navigator.clipboard.writeText(value);
      setAnnouncement(`${label} copied to clipboard.`);
    } catch {
      setAnnouncement(`Clipboard access is unavailable for ${label}. Select the value to copy it manually.`);
    }
  };

  const showPreviewNotice = (title: string, message: string, detail?: string) => {
    setDialog({ title, message, detail });
  };

  const refreshSample = () => {
    setLastUpdated(
      new Date().toLocaleTimeString("en-US", {
        hour: "numeric",
        minute: "2-digit",
      }),
    );
    setAnnouncement("Sample data refreshed. No network query was performed.");
  };

  const runSearch = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const value = query.trim().toLowerCase();
    if (!value) return;

    if (["provider", "warp", "vpn", "wireguard"].some((item) => value.includes(item))) {
      setView("providers");
    } else if (["diagnostic", "route", "dns", "interface", "address"].some((item) => value.includes(item))) {
      setView("diagnostics");
    } else if (["setting", "appearance", "theme"].some((item) => value.includes(item))) {
      setView("settings");
    } else {
      setView("overview");
    }
    setAnnouncement(`Showing the ${viewLabel(value)} sample section.`);
    setQuery("");
  };

  const heading = {
    overview: {
      eyebrow: "Workspace",
      title: "Overview",
      description: "A clear view of this device's network path.",
    },
    providers: {
      eyebrow: "Connections",
      title: "Providers",
      description: "Review connection methods and their capabilities.",
    },
    diagnostics: {
      eyebrow: "Network tools",
      title: "Diagnostics",
      description: "Inspect sample routes, interfaces, and address details.",
    },
    settings: {
      eyebrow: "Preferences",
      title: "Settings",
      description: "Adjust the appearance of the NetShift prototype.",
    },
  }[view];

  return (
    <div className="app-shell">
      <a className="skip-link" href="#main-content">Skip to content</a>

      <aside className="sidebar" aria-label="Application sidebar">
        <div className="brand-lockup">
          <div className="brand-mark" aria-hidden="true">
            <Network size={21} strokeWidth={2.1} />
          </div>
          <div className="brand-copy">
            <span className="brand-name">NetShift</span>
            <span className="brand-caption">NETWORK CONTROL</span>
          </div>
        </div>

        <div className="workspace-switcher">
          <div className="workspace-avatar"><Laptop size={16} aria-hidden="true" /></div>
          <div className="workspace-copy">
            <span>My device</span>
            <small>Windows desktop</small>
          </div>
          <ChevronDown size={15} className="workspace-chevron" aria-hidden="true" />
        </div>

        <nav className="primary-nav" aria-label="Primary navigation">
          <span className="nav-section-label">WORKSPACE</span>
          {navItems.map(({ id, label, icon: Icon }) => (
            <button
              className={view === id ? "nav-item nav-item--active" : "nav-item"}
              type="button"
              key={id}
              onClick={() => setView(id)}
              aria-current={view === id ? "page" : undefined}
              title={label}
            >
              <Icon size={19} strokeWidth={1.9} aria-hidden="true" />
              <span>{label}</span>
              {id === "diagnostics" ? <span className="nav-count">3</span> : null}
            </button>
          ))}
        </nav>

        <div className="sidebar-bottom">
          <div className="sidebar-help">
            <div className="help-icon"><CircleHelp size={17} aria-hidden="true" /></div>
            <div>
              <strong>Need a hand?</strong>
              <span>Explore the demo safely.</span>
            </div>
            <button
              className="icon-button"
              type="button"
              aria-label="About this prototype"
              onClick={() => showPreviewNotice(
                "About this prototype",
                "This is a static, mock-data interface for evaluating the desktop shell.",
                "It does not inspect adapters, routes, DNS, VPN clients, or public IP addresses. Connection-changing actions are disabled.",
              )}
            >
              <ArrowRight size={15} aria-hidden="true" />
            </button>
          </div>
          <div className="sidebar-footer">
            <span className="online-dot" />
            <span>Prototype environment</span>
            <span className="version-label">v0.1.0</span>
          </div>
        </div>
      </aside>

      <div className="workspace">
        <header className="topbar">
          <div className="breadcrumbs">
            <span>NetShift</span>
            <ChevronRight size={14} aria-hidden="true" />
            <strong>{heading.title}</strong>
          </div>

          <div className="topbar-actions">
            <form className="search-box" onSubmit={runSearch} role="search">
              <Search size={17} aria-hidden="true" />
              <input
                type="search"
                value={query}
                onChange={(event) => setQuery(event.currentTarget.value)}
                placeholder="Search NetShift"
                aria-label="Search sample sections"
              />
              <kbd>Enter</kbd>
            </form>
            <span className="topbar-divider" />
            <button
              className="icon-button topbar-icon"
              type="button"
              aria-label={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
              title={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            >
              {theme === "dark" ? <Sun size={18} /> : <Moon size={18} />}
            </button>
            <button
              className="icon-button topbar-icon notification-button"
              type="button"
              aria-label="Notifications"
              onClick={() => showPreviewNotice(
                "You're all caught up",
                "There are no live notifications in this prototype.",
                "Network events and provider state are not being monitored.",
              )}
            >
              <Bell size={18} aria-hidden="true" />
              <span className="notification-dot" />
            </button>
            <div className="profile-avatar" aria-label="Local demo profile">DB</div>
          </div>
        </header>

        <main id="main-content" className="main-content">
          <div className="prototype-banner" role="note">
            <span className="prototype-indicator"><span /> DEMO MODE</span>
            <p>Sample data only. No live network inspection or system changes are enabled.</p>
            <button
              type="button"
              className="banner-link"
              onClick={() => showPreviewNotice(
                "Read-only prototype",
                "Everything displayed in this interface is illustrative sample data.",
                "The shell has no network inspection command, provider action, or system configuration permission.",
              )}
            >
              Safety details <ChevronRight size={14} />
            </button>
          </div>

          <PageHeading
            eyebrow={heading.eyebrow}
            title={heading.title}
            description={heading.description}
            action={
              <div className="heading-actions">
                <span className="last-updated"><Clock3 size={14} /> Updated {lastUpdated}</span>
                <button className="button button--outlined" type="button" onClick={refreshSample}>
                  <RefreshCw size={16} aria-hidden="true" />
                  Refresh sample
                </button>
              </div>
            }
          />

          {view === "overview" ? (
            <OverviewView
              onNavigate={setView}
              onCopy={copyToClipboard}
              onPreview={showPreviewNotice}
            />
          ) : null}
          {view === "providers" ? (
            <ProvidersView onPreview={showPreviewNotice} />
          ) : null}
          {view === "diagnostics" ? (
            <DiagnosticsView onCopy={copyToClipboard} onPreview={showPreviewNotice} />
          ) : null}
          {view === "settings" ? (
            <SettingsView theme={theme} onThemeChange={setTheme} onPreview={showPreviewNotice} />
          ) : null}

          <footer className="content-footer">
            <span>NETSHIFT DESKTOP PROTOTYPE</span>
            <span>Built for Windows · LTR layout · Mock data</span>
          </footer>
        </main>
      </div>

      <div className="sr-only" role="status" aria-live="polite" aria-atomic="true">
        {announcement}
      </div>

      <dialog
        ref={dialogRef}
        className="action-dialog"
        aria-labelledby="dialog-title"
        onClose={() => setDialog(null)}
      >
        <div className="dialog-symbol"><ShieldCheck size={23} aria-hidden="true" /></div>
        <button
          type="button"
          className="icon-button dialog-close"
          aria-label="Close dialog"
          onClick={() => setDialog(null)}
        >
          <X size={18} aria-hidden="true" />
        </button>
        <p className="eyebrow">PROTOTYPE NOTICE</p>
        <h2 id="dialog-title">{dialog?.title ?? "Preview only"}</h2>
        <p className="dialog-message">{dialog?.message}</p>
        {dialog?.detail ? <p className="dialog-detail">{dialog.detail}</p> : null}
        <div className="dialog-actions">
          <button type="button" className="button button--primary" onClick={() => setDialog(null)}>
            Got it
          </button>
        </div>
      </dialog>
    </div>
  );
}

function viewLabel(value: string) {
  if (["provider", "warp", "vpn", "wireguard"].some((item) => value.includes(item))) return "Providers";
  if (["diagnostic", "route", "dns", "interface", "address"].some((item) => value.includes(item))) return "Diagnostics";
  if (["setting", "appearance", "theme"].some((item) => value.includes(item))) return "Settings";
  return "Overview";
}

function OverviewView({
  onNavigate,
  onCopy,
  onPreview,
}: {
  onNavigate: (view: ViewId) => void;
  onCopy: (label: string, value: string) => void;
  onPreview: (title: string, message: string, detail?: string) => void;
}) {
  return (
    <div className="view-stack">
      <section className="overview-hero-grid" aria-label="Connection summary">
        <article className="hero-card">
          <div className="hero-card-top">
            <div>
              <p className="eyebrow">NETWORK STATUS</p>
              <h2>Your connection, at a glance.</h2>
              <p className="hero-description">A sample snapshot of the local network path for this device.</p>
            </div>
            <span className="demo-orbit" aria-hidden="true">
              <span className="orbit-ring orbit-ring--outer" />
              <span className="orbit-ring orbit-ring--inner" />
              <span className="orbit-core"><Network size={31} /></span>
              <span className="orbit-node orbit-node--top" />
              <span className="orbit-node orbit-node--right" />
              <span className="orbit-node orbit-node--bottom" />
            </span>
          </div>
          <div className="hero-status-line">
            <span className="status-pulse" />
            <strong>Direct connection</strong>
            <span className="hero-status-separator" />
            <span>No provider active</span>
          </div>
          <div className="hero-card-bottom">
            <div className="hero-meta">
              <span>ACTIVE INTERFACE</span>
              <strong>Ethernet 2</strong>
            </div>
            <div className="hero-meta">
              <span>ROUTE SCOPE</span>
              <strong>This device</strong>
            </div>
            <button
              type="button"
              className="button button--hero"
              onClick={() => onNavigate("providers")}
            >
              Explore providers <ArrowRight size={16} />
            </button>
          </div>
        </article>

        <article className="surface-card route-card">
          <div className="section-card-heading">
            <div>
              <span className="eyebrow">ROUTE SUMMARY</span>
              <h3>Default route</h3>
            </div>
            <span className="subtle-icon"><Route size={18} /></span>
          </div>
          <div className="route-visual">
            <div className="route-endpoint">
              <span className="route-endpoint-icon"><Laptop size={17} /></span>
              <div><strong>This device</strong><small>Ethernet 2</small></div>
            </div>
            <div className="route-path"><span /><span /><span /></div>
            <div className="route-endpoint">
              <span className="route-endpoint-icon route-endpoint-icon--accent"><Globe size={17} /></span>
              <div><strong>Gateway</strong><small dir="ltr">{sampleValues.gateway}</small></div>
            </div>
          </div>
          <div className="route-card-footer">
            <span>IPv4 default route</span>
            <StatusTag tone="positive">Sample route</StatusTag>
          </div>
          <button type="button" className="text-link" onClick={() => onNavigate("diagnostics")}>
            View route details <ChevronRight size={15} />
          </button>
        </article>
      </section>

      <section className="section-block">
        <div className="section-heading">
          <div>
            <p className="eyebrow">LOCAL SNAPSHOT</p>
            <h2>Network at a glance</h2>
          </div>
          <span className="section-note">Illustrative values · RFC documentation ranges</span>
        </div>
        <div className="metric-grid">
          <MetricCard
            icon={Globe}
            label="Local IPv4 address"
            value={sampleValues.ipv4}
            detail="Assigned to the sample adapter"
            onCopy={onCopy}
            copyLabel="IPv4 address"
          />
          <MetricCard
            icon={Network}
            label="Local IPv6 address"
            value={sampleValues.ipv6}
            detail="Documentation prefix"
            onCopy={onCopy}
            copyLabel="IPv6 address"
          />
          <MetricCard
            icon={Cable}
            label="Network interface"
            value="Ethernet 2"
            detail="ifIndex 12 · Sample adapter"
            onCopy={onCopy}
            copyLabel="Interface identifier"
          />
          <MetricCard
            icon={Route}
            label="Default gateway"
            value={sampleValues.gateway}
            detail="IPv4 default route · Sample"
            onCopy={onCopy}
            copyLabel="Default gateway"
          />
        </div>
      </section>

      <section className="overview-lower-grid">
        <article className="surface-card details-card">
          <div className="section-card-heading">
            <div>
              <p className="eyebrow">INTERFACE DETAILS</p>
              <h3>Ethernet 2</h3>
            </div>
            <StatusTag tone="positive">Sample · Up</StatusTag>
          </div>
          <div className="detail-table">
            <div className="detail-row">
              <span>Interface ID</span>
              <CopyField label="Interface ID" value={sampleValues.interfaceId} onCopy={onCopy} compact />
            </div>
            <div className="detail-row">
              <span>MAC address</span>
              <CopyField label="MAC address" value={sampleValues.mac} onCopy={onCopy} compact />
            </div>
            <div className="detail-row">
              <span>DNS server</span>
              <CopyField label="DNS server" value={sampleValues.dns} onCopy={onCopy} compact />
            </div>
          </div>
          <button type="button" className="text-link" onClick={() => onNavigate("diagnostics")}>
            Open diagnostics <ChevronRight size={15} />
          </button>
        </article>

        <article className="surface-card activity-card">
          <div className="section-card-heading">
            <div>
              <p className="eyebrow">RECENT ACTIVITY</p>
              <h3>What's happening</h3>
            </div>
            <button
              className="icon-button"
              type="button"
              aria-label="Activity information"
              onClick={() => onPreview(
                "Sample activity",
                "This timeline is pre-populated with illustrative events.",
                "The prototype does not subscribe to operating-system events or monitor network traffic.",
              )}
            >
              <MoreHorizontal size={18} />
            </button>
          </div>
          <div className="activity-list">
            {activityItems.map(({ title, description, time, icon: Icon }, index) => (
              <div className="activity-item" key={title}>
                <div className="activity-icon"><Icon size={16} aria-hidden="true" /></div>
                <div className="activity-copy">
                  <strong>{title}</strong>
                  <span>{description}</span>
                </div>
                <time>{time}</time>
                {index !== activityItems.length - 1 ? <span className="activity-connector" /> : null}
              </div>
            ))}
          </div>
        </article>
      </section>
    </div>
  );
}

function ProvidersView({
  onPreview,
}: {
  onPreview: (title: string, message: string, detail?: string) => void;
}) {
  return (
    <div className="view-stack">
      <div className="providers-intro surface-card">
        <div className="providers-intro-icon"><ShieldCheck size={23} /></div>
        <div>
          <h2>One place for every connection method.</h2>
          <p>Compare provider capabilities before choosing how this device connects.</p>
        </div>
        <StatusTag>Demo catalog</StatusTag>
      </div>
      <div className="provider-grid">
        {providerItems.map(({ name, description, badge, icon: Icon, detail }, index) => (
          <article className="surface-card provider-card" key={name}>
            <div className="provider-card-top">
              <div className={`provider-logo provider-logo--${index + 1}`}><Icon size={23} /></div>
              <button
                type="button"
                className="icon-button"
                aria-label={`More about ${name}`}
                onClick={() => onPreview(name, detail)}
              >
                <MoreHorizontal size={18} />
              </button>
            </div>
            <h3>{name}</h3>
            <p>{description}</p>
            <div className="provider-capabilities">
              <span><Check size={13} /> IPv4</span>
              <span><Check size={13} /> Device scope</span>
              {index === 0 ? <span><Check size={13} /> IPv6</span> : null}
            </div>
            <div className="provider-card-bottom">
              <StatusTag>{badge}</StatusTag>
              <button
                type="button"
                className="button button--tonal"
                onClick={() => onPreview(
                  "Connection actions are disabled",
                  `${name} is displayed for layout evaluation only.`,
                  "No provider APIs, services, executables, tunnels, or network configuration are accessed by this button.",
                )}
              >
                Preview details <ArrowRight size={15} />
              </button>
            </div>
          </article>
        ))}
      </div>
      <div className="notice-card">
        <CircleAlert size={18} />
        <p><strong>Nothing is connected.</strong> The provider catalog is illustrative. No provider has been detected or configured on this device.</p>
      </div>
    </div>
  );
}

function DiagnosticsView({
  onCopy,
  onPreview,
}: {
  onCopy: (label: string, value: string) => void;
  onPreview: (title: string, message: string, detail?: string) => void;
}) {
  const diagnostics = [
    { title: "Adapter information", description: "Local interface and hardware identifier.", value: sampleValues.interfaceId, icon: Cable },
    { title: "IPv4 configuration", description: "Local address and default gateway.", value: sampleValues.ipv4, icon: Globe },
    { title: "IPv6 configuration", description: "Local documentation address.", value: sampleValues.ipv6, icon: Network },
    { title: "DNS configuration", description: "Sample resolver assigned to the demo interface.", value: sampleValues.dns, icon: Server },
    { title: "Default route", description: "Sample route used to leave the local subnet.", value: sampleValues.gateway, icon: Route },
    { title: "Service port", description: "Example HTTPS destination port.", value: sampleValues.port, icon: LockKeyhole },
    { title: "Diagnostic log line", description: "Illustrative text line containing sample network values.", value: sampleValues.log, icon: FileText },
    { title: "Log file location", description: "Illustrative Windows path for a future diagnostics export.", value: sampleValues.path, icon: FileText },
  ];

  return (
    <div className="view-stack">
      <div className="diagnostics-toolbar">
        <div>
          <h2>Diagnostic snapshot</h2>
          <p>Each result is a fixture, not a live test.</p>
        </div>
        <button
          type="button"
          className="button button--primary"
          onClick={() => onPreview(
            "Diagnostics are sample-only",
            "The diagnostic runner is not wired to the operating system.",
            "This action does not send probes, resolve DNS, inspect routes, read logs, or change network state.",
          )}
        >
          <Activity size={16} /> Run preview
        </button>
      </div>

      <div className="diagnostic-grid">
        <article className="surface-card diagnostic-summary">
          <div className="diagnostic-score">
            <div className="score-ring"><span>--</span><small>DEMO</small></div>
          </div>
          <h3>No live test results</h3>
          <p>Real connectivity health is not measured by this prototype.</p>
          <div className="diagnostic-summary-footer">
            <span><span className="legend-dot legend-dot--sample" /> Sample fixture</span>
            <span>Last refreshed manually</span>
          </div>
        </article>

        <article className="surface-card diagnostic-results">
          <div className="section-card-heading">
            <div><p className="eyebrow">CONFIGURATION FIXTURES</p><h3>Sample details</h3></div>
            <StatusTag>Not measured</StatusTag>
          </div>
          <div className="diagnostic-list">
            {diagnostics.map(({ title, description, value, icon: Icon }) => (
              <div className="diagnostic-row" key={title}>
                <span className="diagnostic-row-icon"><Icon size={17} /></span>
                <div className="diagnostic-row-copy"><strong>{title}</strong><span>{description}</span></div>
                <code dir="ltr">{value}</code>
                <button
                  className="icon-button"
                  type="button"
                  aria-label={`Copy ${title}`}
                  onClick={() => onCopy(title, value)}
                >
                  <Copy size={15} />
                </button>
              </div>
            ))}
          </div>
        </article>
      </div>

      <div className="notice-card">
        <ShieldCheck size={18} />
        <p><strong>Read-only by design.</strong> The diagnostic screen has no network command, process launcher, or file reader attached to it.</p>
      </div>
    </div>
  );
}

function SettingsView({
  theme,
  onThemeChange,
  onPreview,
}: {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  onPreview: (title: string, message: string, detail?: string) => void;
}) {
  return (
    <div className="view-stack settings-layout">
      <section className="surface-card settings-card">
        <div className="section-card-heading">
          <div><p className="eyebrow">APPEARANCE</p><h3>Make it yours</h3></div>
          <span className="subtle-icon"><SlidersHorizontal size={18} /></span>
        </div>
        <p className="settings-description">Choose the surface theme used by the prototype.</p>
        <div className="theme-options" role="radiogroup" aria-label="Color theme">
          <button
            type="button"
            className={theme === "dark" ? "theme-option theme-option--selected" : "theme-option"}
            role="radio"
            aria-checked={theme === "dark"}
            onClick={() => onThemeChange("dark")}
          >
            <span className="theme-preview theme-preview--dark"><i /><i /><i /></span>
            <span><strong>Dark</strong><small>Low-light workspace</small></span>
            {theme === "dark" ? <Check size={17} /> : null}
          </button>
          <button
            type="button"
            className={theme === "light" ? "theme-option theme-option--selected" : "theme-option"}
            role="radio"
            aria-checked={theme === "light"}
            onClick={() => onThemeChange("light")}
          >
            <span className="theme-preview theme-preview--light"><i /><i /><i /></span>
            <span><strong>Light</strong><small>Bright workspace</small></span>
            {theme === "light" ? <Check size={17} /> : null}
          </button>
        </div>
      </section>

      <section className="surface-card settings-card">
        <div className="section-card-heading">
          <div><p className="eyebrow">APPLICATION</p><h3>Prototype information</h3></div>
          <span className="subtle-icon"><FileText size={18} /></span>
        </div>
        <div className="settings-list">
          <div className="settings-row"><span>Interface language</span><strong>English</strong></div>
          <div className="settings-row"><span>Layout direction</span><strong>Left to right (LTR)</strong></div>
          <div className="settings-row"><span>Data source</span><strong>Static sample data</strong></div>
          <div className="settings-row"><span>Network permissions</span><StatusTag tone="positive">Not exposed</StatusTag></div>
          <div className="settings-row"><span>Application version</span><strong>0.1.0 · Prototype</strong></div>
        </div>
        <button
          type="button"
          className="text-link"
          onClick={() => onPreview(
            "Prototype security boundary",
            "The frontend is a static shell and contains no network or provider commands.",
            "The Tauri backend registers no invoke handlers, the capability list is empty, and this prototype uses sample data only.",
          )}
        >
          Review safety boundary <ChevronRight size={15} />
        </button>
      </section>
    </div>
  );
}

export default App;
