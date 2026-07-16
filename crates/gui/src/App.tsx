import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { SkipToContent } from "@entur/a11y";
import { BannerAlertBox } from "@entur/alert";
import * as api from "./api";
import type {
  Catalog,
  Config,
  FixProposal,
  LintError,
  PipelineEvent,
  ValidateReport,
} from "./types";
import { phaseKey } from "./types";
import { useColorMode } from "./hooks";
import Toolbar from "./components/Toolbar";
import ConfigPanel from "./components/ConfigPanel";
import EditorPane from "./components/EditorPane";
import PipelinePanel from "./components/PipelinePanel";
import FixModal from "./components/FixModal";
import UrlModal from "./components/UrlModal";

export interface PhaseStatus {
  key: string;
  status: "running" | "ok" | "fail";
}

export interface OpenFile {
  path: string;
  content: string;
  readOnly: boolean;
}

const MAX_LOG_LINES = 5000;

export default function App() {
  const colorMode = useColorMode();
  const [root, setRoot] = useState<string | null>(null);
  const [specs, setSpecs] = useState<string[]>([]);
  const [specPath, setSpecPath] = useState<string | null>(null);
  const [file, setFile] = useState<OpenFile | null>(null);
  const [config, setConfig] = useState<Config | null>(null);
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [dockerError, setDockerError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [phases, setPhases] = useState<PhaseStatus[]>([]);
  const [logs, setLogs] = useState<string[]>([]);
  const [lintErrors, setLintErrors] = useState<LintError[]>([]);
  const [report, setReport] = useState<ValidateReport | null>(null);
  const [banner, setBanner] = useState<string | null>(null);
  const [proposal, setProposal] = useState<FixProposal | null>(null);
  const [urlModalOpen, setUrlModalOpen] = useState(false);

  const showError = useCallback((e: unknown) => {
    setBanner(String(e));
  }, []);

  const openSpec = useCallback(
    async (workspace: string, path: string) => {
      try {
        const content = await api.readWorkspaceFile(workspace, path);
        setSpecPath(path);
        setFile({ path, content, readOnly: false });
      } catch (e) {
        showError(e);
      }
    },
    [showError],
  );

  const loadWorkspace = useCallback(
    async (dir: string) => {
      setBanner(null);
      setRoot(dir);
      setFile(null);
      setSpecPath(null);
      setPhases([]);
      setLogs([]);
      setLintErrors([]);
      try {
        const cfg = await api.loadConfig(dir);
        setConfig(cfg);
        setCatalog(await api.generatorCatalog(dir));
        setReport(await api.loadReport(dir));
        const found = await api.discoverSpecs(dir);
        setSpecs(found);
        const initial = cfg.spec ?? found[0];
        if (initial) await openSpec(dir, initial);
        await api.dockerStatus(cfg.compile);
        setDockerError(null);
      } catch (e) {
        // Config/spec loading errors are fatal for the workspace; docker
        // unavailability is not, so only flag it.
        if (config === null && catalog === null) showError(e);
        else setDockerError(String(e));
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [openSpec, showError],
  );

  const handleOpenFolder = useCallback(async () => {
    const dir = await open({ directory: true });
    if (typeof dir === "string") await loadWorkspace(dir);
  }, [loadWorkspace]);

  const handleFetchUrl = useCallback(
    async (url: string) => {
      if (!root) return;
      const fetched = await api.fetchSpecUrl(root, url);
      setSpecs((prev) => (prev.includes(fetched) ? prev : [...prev, fetched]));
      await openSpec(root, fetched);
      setUrlModalOpen(false);
    },
    [root, openSpec],
  );

  const updateConfig = useCallback(
    async (next: Config) => {
      setConfig(next);
      if (!root) return;
      try {
        await api.saveConfig(root, next);
      } catch (e) {
        showError(e);
      }
    },
    [root, showError],
  );

  const handleEvent = useCallback((event: PipelineEvent) => {
    switch (event.type) {
      case "phase_started": {
        const key = phaseKey(event.data);
        setPhases((prev) => [
          ...prev.filter((p) => p.key !== key),
          { key, status: "running" },
        ]);
        break;
      }
      case "log":
        setLogs((prev) =>
          [...prev, `[${phaseKey(event.data.phase)}] ${event.data.line}`].slice(
            -MAX_LOG_LINES,
          ),
        );
        break;
      case "phase_finished": {
        const key = phaseKey(event.data.phase);
        const status = event.data.success ? "ok" : "fail";
        setPhases((prev) =>
          prev.map((p) => (p.key === key ? { ...p, status } : p)),
        );
        break;
      }
      case "lint_completed":
        api.parseLint(event.data.log).then(setLintErrors).catch(() => {});
        break;
      case "completed":
        setReport(event.data);
        setRunning(false);
        break;
      case "aborted":
        setLogs((prev) => [...prev, `aborted: ${event.data}`]);
        setRunning(false);
        break;
    }
  }, []);

  const handleRun = useCallback(async () => {
    if (!root || !specPath) return;
    setBanner(null);
    setPhases([]);
    setLogs([]);
    setLintErrors([]);
    setReport(null);
    setRunning(true);
    try {
      await api.startPipeline(root, specPath, handleEvent);
    } catch (e) {
      setRunning(false);
      showError(e);
    }
  }, [root, specPath, handleEvent, showError]);

  const handleCancel = useCallback(() => {
    api.cancelPipeline().catch(showError);
  }, [showError]);

  const handleSaveFile = useCallback(
    async (content: string) => {
      if (!root || !file || file.readOnly) return;
      try {
        await api.writeWorkspaceFile(root, file.path, content);
        setFile({ ...file, content });
      } catch (e) {
        showError(e);
      }
    },
    [root, file, showError],
  );

  const handleOpenGenerated = useCallback(
    async (path: string) => {
      if (!root) return;
      try {
        const content = await api.readWorkspaceFile(root, path);
        setFile({ path, content, readOnly: true });
      } catch (e) {
        showError(e);
      }
    },
    [root, showError],
  );

  const handleBackToSpec = useCallback(async () => {
    if (root && specPath) await openSpec(root, specPath);
  }, [root, specPath, openSpec]);

  const handleFix = useCallback(
    async (error: LintError) => {
      if (!root || !specPath) return;
      try {
        const found = await api.proposeFix(root, specPath, error);
        if (!found) {
          setBanner(`No automatic fix available for rule '${error.rule}'`);
          return;
        }
        setProposal(found);
      } catch (e) {
        showError(e);
      }
    },
    [root, specPath, showError],
  );

  const handleApplyFix = useCallback(async () => {
    if (!root || !specPath || !proposal) return;
    try {
      await api.applyFix(root, specPath, proposal);
      setProposal(null);
      await openSpec(root, specPath);
    } catch (e) {
      setProposal(null);
      showError(e);
    }
  }, [root, specPath, proposal, openSpec, showError]);

  // Re-check docker when the compile toggle changes.
  useEffect(() => {
    if (!config) return;
    api
      .dockerStatus(config.compile)
      .then(() => setDockerError(null))
      .catch((e) => setDockerError(String(e)));
  }, [config?.compile]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="app">
      <SkipToContent>Skip to editor</SkipToContent>
      <Toolbar
        root={root}
        specs={specs}
        specPath={specPath}
        running={running}
        dockerError={dockerError}
        onOpenFolder={handleOpenFolder}
        onOpenUrl={() => setUrlModalOpen(true)}
        onSelectSpec={(p) => root && openSpec(root, p)}
        onRun={handleRun}
        onCancel={handleCancel}
      />
      {banner && (
        <BannerAlertBox
          variant="negative"
          closable
          onClose={() => setBanner(null)}
        >
          {banner}
        </BannerAlertBox>
      )}
      <div className="app__body">
        {config && catalog ? (
          <ConfigPanel config={config} catalog={catalog} onChange={updateConfig} />
        ) : (
          <aside className="sidebar sidebar--empty">Open a folder to begin</aside>
        )}
        <EditorPane
          file={file}
          colorMode={colorMode}
          lintErrors={file && !file.readOnly ? lintErrors : []}
          onSave={handleSaveFile}
          onBackToSpec={file?.readOnly ? handleBackToSpec : undefined}
        />
        <PipelinePanel
          phases={phases}
          logs={logs}
          lintErrors={lintErrors}
          report={report}
          running={running}
          onFix={handleFix}
          onOpenGenerated={handleOpenGenerated}
          listGenerated={(scope, generator) =>
            root ? api.listGeneratedFiles(root, scope, generator) : Promise.resolve([])
          }
        />
      </div>
      <FixModal
        proposal={proposal}
        onApply={handleApplyFix}
        onDismiss={() => setProposal(null)}
      />
      <UrlModal
        open={urlModalOpen}
        onDismiss={() => setUrlModalOpen(false)}
        onFetch={handleFetchUrl}
      />
    </div>
  );
}
