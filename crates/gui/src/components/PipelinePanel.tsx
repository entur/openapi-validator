import { useEffect, useRef, useState } from "react";
import { Tabs, TabList, Tab, TabPanels, TabPanel } from "@entur/tab";
import {
  Table,
  TableHead,
  TableBody,
  TableRow,
  HeaderCell,
  DataCell,
} from "@entur/table";
import { StatusBadge } from "@entur/layout";
import { TertiaryButton } from "@entur/button";
import { Loader } from "@entur/loader";
import { Heading5, Link, Paragraph, SmallText } from "@entur/typography";
import { VisuallyHidden } from "@entur/a11y";
import type { LintError, Severity, ValidateReport } from "../types";
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

const phaseVariant = { running: "information", ok: "success", fail: "negative" } as const;

const severityVariant: Record<Severity, "negative" | "warning" | "information" | "neutral"> = {
  error: "negative",
  warning: "warning",
  info: "information",
  hint: "neutral",
};

export default function PipelinePanel(props: Props) {
  const [tabIndex, setTabIndex] = useState(0);
  const [files, setFiles] = useState<string[]>([]);
  const [browsing, setBrowsing] = useState<string | null>(null);
  const logEnd = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (tabIndex === 0) logEnd.current?.scrollIntoView({ behavior: "instant" });
  }, [props.logs.length, tabIndex]);

  useEffect(() => {
    if (props.running) setTabIndex(0);
    else if (props.report) setTabIndex(2);
  }, [props.running, props.report]);

  useEffect(() => {
    if (props.lintErrors.length > 0 && !props.running) setTabIndex(1);
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
    <section className="pipeline" aria-label="Validation results">
      <Tabs index={tabIndex} onChange={setTabIndex}>
        <TabList>
          <Tab>Progress</Tab>
          <Tab>Lint ({props.lintErrors.length})</Tab>
          <Tab>Results</Tab>
        </TabList>
        <TabPanels className="pipeline__panels">
          <TabPanel className="pipeline__panel">
            {props.running && props.phases.length === 0 && <Loader>Starting…</Loader>}
            {props.phases.length > 0 && (
              <ul className="pipeline__phases">
                {props.phases.map((p) => (
                  <li key={p.key}>
                    <StatusBadge variant={phaseVariant[p.status]}>{p.key}</StatusBadge>
                  </li>
                ))}
              </ul>
            )}
            <pre className="pipeline__log">
              {props.logs.join("\n")}
              <div ref={logEnd} />
            </pre>
          </TabPanel>

          <TabPanel className="pipeline__panel">
            {props.lintErrors.length === 0 ? (
              <Paragraph className="pipeline__empty">No lint findings.</Paragraph>
            ) : (
              <Table spacing="small" fixed>
                <TableHead>
                  <TableRow>
                    <HeaderCell>Line</HeaderCell>
                    <HeaderCell>Severity</HeaderCell>
                    <HeaderCell>Message</HeaderCell>
                    <HeaderCell>
                      <VisuallyHidden>Actions</VisuallyHidden>
                    </HeaderCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {props.lintErrors.map((e, i) => (
                    <TableRow key={i}>
                      <DataCell>{e.line}</DataCell>
                      <DataCell>
                        <StatusBadge variant={severityVariant[e.severity]}>
                          {e.severity}
                        </StatusBadge>
                      </DataCell>
                      <DataCell>
                        {e.message}
                        <br />
                        <SmallText>{e.rule}</SmallText>
                      </DataCell>
                      <DataCell>
                        <TertiaryButton onClick={() => props.onFix(e)}>
                          Fix
                        </TertiaryButton>
                      </DataCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </TabPanel>

          <TabPanel className="pipeline__panel">
            {!props.report ? (
              <Paragraph className="pipeline__empty">
                No report yet. Run a validation.
              </Paragraph>
            ) : (
              <div className="pipeline__report">
                <Paragraph>
                  <strong>{props.report.spec}</strong> — {props.report.summary.passed}/
                  {props.report.summary.total} passed
                </Paragraph>
                {props.report.phases.lint && (
                  <Paragraph>
                    Lint ({props.report.phases.lint.linter}):{" "}
                    <StatusBadge
                      variant={
                        props.report.phases.lint.status === "passed"
                          ? "success"
                          : "negative"
                      }
                    >
                      {props.report.phases.lint.status}
                    </StatusBadge>
                  </Paragraph>
                )}
                {(["generate", "compile"] as const).map((phase) => {
                  const steps = props.report!.phases[phase];
                  if (!steps) return null;
                  return (
                    <div key={phase}>
                      <Heading5 as="h3">{phase}</Heading5>
                      <ul className="pipeline__steps">
                        {steps.map((s) => (
                          <li key={`${phase}/${s.scope}/${s.generator}`}>
                            <StatusBadge
                              variant={s.status === "passed" ? "success" : "negative"}
                            >
                              {s.scope}/{s.generator}
                            </StatusBadge>
                            {phase === "generate" && (
                              <TertiaryButton
                                onClick={() => browse(s.scope, s.generator)}
                              >
                                {browsing === `${s.scope}/${s.generator}`
                                  ? "Hide files"
                                  : "Browse files"}
                              </TertiaryButton>
                            )}
                          </li>
                        ))}
                      </ul>
                    </div>
                  );
                })}
                {browsing && (
                  <ul className="pipeline__files">
                    {files.map((f) => (
                      <li key={f}>
                        <Link as="button" onClick={() => props.onOpenGenerated(f)}>
                          {f}
                        </Link>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}
          </TabPanel>
        </TabPanels>
      </Tabs>
    </section>
  );
}
