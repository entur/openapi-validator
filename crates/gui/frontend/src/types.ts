// Mirrors of the oav-lib serde types crossing the IPC boundary.

export type Mode = "server" | "client" | "both";
export type Linter = "spectral" | "redocly" | "none";
export type Jobs = "auto" | number;

export interface Config {
  spec: string | null;
  mode: Mode;
  lint: boolean;
  generate: boolean;
  compile: boolean;
  server_generators: string[];
  client_generators: string[];
  generator_overrides: Record<string, string>;
  generator_image: string;
  redocly_image: string;
  linter: Linter;
  spectral_image: string;
  spectral_ruleset: string;
  spectral_fail_severity: string;
  manage_gitignore: boolean;
  custom_generators_dir: string | null;
  docker_timeout: number;
  search_depth: number;
  jobs: Jobs;
  keys?: Record<string, string[]>;
}

export interface GeneratorDef {
  name: string;
  scope: "server" | "client";
  config_yaml: string;
}

export interface CustomGeneratorDef {
  name: string;
  scope: string;
  generate: { image: string; command: string };
  compile: { image: string; command: string } | null;
}

export interface Catalog {
  builtin: GeneratorDef[];
  custom: CustomGeneratorDef[];
}

export type Severity = "error" | "warning" | "info" | "hint";

export interface LintError {
  line: number;
  col: number;
  severity: Severity;
  rule: string;
  message: string;
  json_path: string | null;
}

export interface CompileError {
  file: string;
  line: number;
  message: string;
}

export interface FixProposal {
  rule: string;
  description: string;
  target_line: number;
  context_before: string[];
  inserted: string[];
  context_after: string[];
}

export interface LintResult {
  linter: string;
  status: string;
  log: string;
}

export interface StepResult {
  generator: string;
  scope: string;
  status: string;
  log: string;
}

export interface ValidateReport {
  spec: string;
  mode: string;
  phases: {
    lint: LintResult | null;
    generate: StepResult[] | null;
    compile: StepResult[] | null;
  };
  summary: { total: number; passed: number; failed: number };
}

export type Phase =
  | { type: "lint" }
  | { type: "generate"; generator: string; scope: string }
  | { type: "compile"; generator: string; scope: string };

export type PipelineEvent =
  | { type: "phase_started"; data: Phase }
  | { type: "log"; data: { phase: Phase; line: string } }
  | { type: "phase_finished"; data: { phase: Phase; success: boolean } }
  | { type: "lint_completed"; data: LintResult }
  | { type: "completed"; data: ValidateReport }
  | { type: "aborted"; data: string };

export function phaseKey(phase: Phase): string {
  return phase.type === "lint"
    ? "lint"
    : `${phase.type} ${phase.scope}/${phase.generator}`;
}
