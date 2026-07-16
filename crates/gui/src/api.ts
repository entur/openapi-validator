import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  Catalog,
  CompileError,
  Config,
  FixProposal,
  LintError,
  PipelineEvent,
  ValidateReport,
} from "./types";

export const discoverSpecs = (root: string) =>
  invoke<string[]>("discover_specs", { root });

export const loadConfig = (root: string) =>
  invoke<Config>("load_config", { root });

export const saveConfig = (root: string, config: Config) =>
  invoke<void>("save_config", { root, config });

export const validateConfig = (root: string, config: Config) =>
  invoke<string[]>("validate_config", { root, config });

export const generatorCatalog = (root: string) =>
  invoke<Catalog>("generator_catalog", { root });

export const dockerStatus = (compose: boolean) =>
  invoke<void>("docker_status", { compose });

export const readWorkspaceFile = (root: string, path: string) =>
  invoke<string>("read_workspace_file", { root, path });

export const writeWorkspaceFile = (root: string, path: string, content: string) =>
  invoke<void>("write_workspace_file", { root, path, content });

export const listGeneratedFiles = (root: string, scope: string, generator: string) =>
  invoke<string[]>("list_generated_files", { root, scope, generator });

export const parseLint = (log: string) =>
  invoke<LintError[]>("parse_lint", { log });

export const parseCompile = (log: string) =>
  invoke<CompileError[]>("parse_compile", { log });

export const loadReport = (root: string) =>
  invoke<ValidateReport | null>("load_report", { root });

export const proposeFix = (root: string, specPath: string, error: LintError) =>
  invoke<FixProposal | null>("propose_fix", { root, specPath, error });

export const applyFix = (root: string, specPath: string, proposal: FixProposal) =>
  invoke<void>("apply_fix", { root, specPath, proposal });

export const fetchSpecUrl = (root: string, url: string) =>
  invoke<string>("fetch_spec_url", { root, url });

export const startPipeline = (
  root: string,
  specPath: string,
  onEvent: (event: PipelineEvent) => void,
) => {
  const channel = new Channel<PipelineEvent>();
  channel.onmessage = onEvent;
  return invoke<void>("start_pipeline", { root, specPath, onEvent: channel });
};

export const cancelPipeline = () => invoke<void>("cancel_pipeline");
