import { Fragment } from 'react'
import { Link } from 'react-router-dom'
import {
  ARCHITECTURE,
  CTA,
  HERO,
  METRICS,
  NEXT_MILESTONE,
  PROTOCOLS,
  RELEASE,
  ROADMAP,
  SCENARIOS,
  SECURITY_CLOSED,
  SECURITY_OPEN,
  VISION,
} from '../lib/progressData'

export default function ProgressPage() {
  return (
    <div className="pp">
      <div className="pp-inner">
        <header className="pp-hero">
          <div>
            <h1 className="pp-title">{HERO.title}</h1>
            <p className="pp-sub">{HERO.subtitle}</p>
          </div>
          <div className="pp-pill">
            {RELEASE.branch} @ {RELEASE.commit} · {RELEASE.date} · CI {RELEASE.ci}
          </div>
        </header>

        <h2 className="pp-h2">Vision</h2>
        <section className="pp-vision">
          <p className="pp-pitch">{VISION.pitch}</p>
          <div className="pp-chain">
            {VISION.chain.map((node, i) => (
              <Fragment key={node}>
                {i > 0 && <span className="pp-chain-arrow">→</span>}
                <span className="pp-chain-node">{node}</span>
              </Fragment>
            ))}
          </div>
          <div className="pp-callout">
            <strong>{VISION.calloutTitle}</strong> — {VISION.calloutBody}
          </div>
        </section>

        <h2 className="pp-h2">Architecture</h2>
        <section className="pp-arch">
          {ARCHITECTURE.map((layer) => (
            <div className="pp-card" key={layer.title}>
              <div className="pp-layer-title">{layer.title}</div>
              <div className="pp-rfc-note">{layer.body}</div>
              <div className="pp-tags">
                {layer.tags.map((t) => (
                  <span className="pp-tag pp-tag-mono" key={t}>
                    {t}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </section>

        <h2 className="pp-h2">Flagship scenarios</h2>
        <section className="pp-scenarios">
          {SCENARIOS.map((s) => (
            <div className="pp-card" key={s.title}>
              <div className="pp-layer-title">{s.title}</div>
              <ol className="pp-steps">
                {s.steps.map((step) => (
                  <li key={step}>{step}</li>
                ))}
              </ol>
              <div className="pp-ref">{s.ref}</div>
            </div>
          ))}
        </section>

        <h2 className="pp-h2">Live product</h2>
        <section className="pp-card pp-cta">
          <div className="pp-cta-text">
            <h3 className="pp-next-title">{CTA.title}</h3>
            <p className="pp-next-body">{CTA.body}</p>
            <ul className="pp-findings pp-closed pp-cta-list">
              {CTA.bullets.map((b) => (
                <li className="pp-finding" key={b}>
                  <span>{b}</span>
                </li>
              ))}
            </ul>
          </div>
          <div className="pp-cta-action">
            <Link to="/" className="btn btn-primary pp-cta-btn">
              Open the Globe →
            </Link>
          </div>
        </section>

        <h2 className="pp-h2">Progress in numbers</h2>
        <section className="pp-grid">
          {METRICS.map((m) => (
            <div className="pp-card" key={m.label}>
              <div className="pp-metric-value">{m.value}</div>
              <div className="pp-metric-label">{m.label}</div>
              <div className="pp-metric-detail">{m.detail}</div>
            </div>
          ))}
        </section>

        <h2 className="pp-h2">Roadmap</h2>
        <section className="pp-roadmap">
          {ROADMAP.map((group) => (
            <div className="pp-group" key={group.title}>
              <h3 className="pp-group-title">{group.title}</h3>
              <ul className="pp-list">
                {group.items.map((item) => (
                  <li
                    className={`pp-item ${item.done ? 'pp-done' : 'pp-todo'}`}
                    key={item.label}
                  >
                    <span className="pp-mark">{item.done ? '✓' : '○'}</span>
                    <span>{item.label}</span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </section>

        <h2 className="pp-h2">Protocol catalog</h2>
        <section className="pp-protocols">
          {PROTOCOLS.map((p) => (
            <div className="pp-card" key={p.rfc}>
              <div className="pp-rfc-head">
                <span className="pp-rfc">{p.rfc}</span>
                <span
                  className={`pp-badge ${p.delivered ? 'pp-badge-ok' : 'pp-badge-next'}`}
                >
                  {p.status}
                </span>
              </div>
              <div className="pp-rfc-name">{p.name}</div>
              <div className="pp-rfc-note">{p.note}</div>
            </div>
          ))}
        </section>

        <h2 className="pp-h2">Security posture &amp; next milestone</h2>
        <section className="pp-cols">
          <div className="pp-card">
            <div className="pp-group-title">Closed</div>
            <ul className="pp-findings pp-closed">
              {SECURITY_CLOSED.map((f) => (
                <li className="pp-finding" key={f.label}>
                  <span>{f.label}</span>
                </li>
              ))}
            </ul>
            <div className="pp-group-title" style={{ marginTop: 16 }}>
              Open pre-mainnet
            </div>
            <ul className="pp-findings pp-open">
              {SECURITY_OPEN.map((f) => (
                <li className="pp-finding" key={f.label}>
                  <span>{f.label}</span>
                </li>
              ))}
            </ul>
          </div>
          <div className="pp-card">
            <div className="pp-group-title">Next milestone</div>
            <h3 className="pp-next-title">{NEXT_MILESTONE.title}</h3>
            <p className="pp-next-body">{NEXT_MILESTONE.body}</p>
            <div className="pp-tags">
              {NEXT_MILESTONE.tags.map((t) => (
                <span className="pp-tag" key={t}>
                  {t}
                </span>
              ))}
            </div>
          </div>
        </section>

        <footer className="pp-foot">
          Figures generated from source — `make idl` (IDL counts), `make test-fast`
          + integration suites (tests), GitHub Actions (CI). Verified {RELEASE.date} against{' '}
          {RELEASE.branch} = {RELEASE.commit}. Protocol specs live in docs/rfc-003…rfc-012.
        </footer>
      </div>
    </div>
  )
}
