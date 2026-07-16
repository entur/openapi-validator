import { useEffect, useRef, useState } from "react";
import type { LintError, ValidateReport } from "../types";
import type { PhaseStatus } from "../App";

interface Props {
  phases: PhaseStatus[];
  logs: string[];
  lintErrors: LintError[];
  report: ValidateReport | null;
  running: boolean;
  onFix: (error: LintError) => void;
  onOpenGenerated: (path: string) => void;
  listGenerated: (scope: string, generator: string) => Promise<string[]>;
}

type Tab = "progress" | "lint" | "results";

const statusIcon = { running: "⏳", ok: "✅", fail: "❌" } as const;

export default function PipelinePanel(props: Props) {
  const [tab, setTab] = useState<Tab>("progress");
  const [files, setFiles] = useState<string[]>([]);
  const [browsing, setBrowsing] = useState<string | null>(null);
  const logEnd = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (tab === "progress") logEnd.current?.scrollIntoView({ behavior: "instant" });
  }, [props.logs.length, tab]);

  useEffect(() => {
    if (props.running) setTab("progress");
    else if (props.report) setTab("results");
  }, [props.running, props.report]);

  useEffect(() => {
    if (props.lintErrors.length > 0 && !props.running) setTab("lint");
  }, [props.lintErrors, props.running]);

  const browse = async (scope: string, generator: string) => {
    const key = `${scope}/${generator}`;
    if (browsing === key) {
      setBrowsing(null);
      setFiles([]);
      return;
    }
    setBrowsing(key);
    setFiles(await props.listGenerated(scope, generator));
  };

  return (
    <section className="pipeline-panel">
      <nav className="tabs">
        {(["progress", "lint", "results"] as Tab[]).map((t) => (
          <button key={t} className={tab === t ? "active" : ""} onClick={() => setTab(t)}>
            {t === "lint" ? `lint (${props.lintErrors.length})` : t}
          </button>
        ))}
      </nav>

      {tab === "progress" && (
        <div className="progress">
          <ul className="phase-list">
            {props.phases.map((p) => (
              <li key={p.key}>
                {statusIcon[p.status]} {p.key}
              </li>
            ))}
          </ul>
          <pre className="log">
            {props.logs.join("\n")}
            <div ref={logEnd} />
          </pre>
        </div>
      )}

      {tab === "lint" && (
        <div className="lint-table">
          {props.lintErrors.length === 0 ? (
            <p className="muted">No lint findings.</p>
          ) : (
            <table>
              <thead>
                <tr>
                  <th>Line</th>
                  <th>Severity</th>
                  <th>Rule</th>
                  <th>Message</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {props.lintErrors.map((e, i) => (
                  <tr key={i} className={`sev-${e.severity}`}>
                    <td>{e.line}</td>
                    <td>{e.severity}</td>
                    <td>{e.rule}</td>
                    <td>{e.message}</td>
                    <td>
                      <button onClick={() => props.onFix(e)}>Fix</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}

      {tab === "results" && (
        <div className="results">
          {!props.report ? (
            <p className="muted">No report yet. Run a validation.</p>
          ) : (
            <>
              <p>
                <strong>{props.report.spec}</strong> — {props.report.summary.passed}/
                {props.report.summary.total} passed
              </p>
              {props.report.phases.lint && (
                <p>
                  lint ({props.report.phases.lint.linter}):{" "}
                  {props.report.phases.lint.status}
                </p>
              )}
              {(["generate", "compile"] as const).map((phase) => {
                const steps = props.report!.phases[phase];
                if (!steps) return null;
                return (
                  <div key={phase}>
                    <h4>{phase}</h4>
                    <ul className="step-list">
                      {steps.map((s) => (
                        <li key={`${phase}/${s.scope}/${s.generator}`}>
                          {s.status === "passed" ? "✅" : "❌"} {s.scope}/{s.generator}
                          {phase === "generate" && (
                            <button onClick={() => browse(s.scope, s.generator)}>
                              {browsing === `${s.scope}/${s.generator}`
                                ? "hide files"
                                : "browse files"}
                            </button>
                          )}
                        </li>
                      ))}
                    </ul>
                  </div>
                );
              })}
              {browsing && (
                <ul className="file-list">
                  {files.map((f) => (
                    <li key={f}>
                      <a onClick={() => props.onOpenGenerated(f)}>{f}</a>
                    </li>
                  ))}
                </ul>
              )}
            </>
          )}
        </div>
      )}
    </section>
  );
}
