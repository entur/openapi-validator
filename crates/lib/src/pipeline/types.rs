use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::custom::CustomGeneratorDef;

/// Mirrors the CLI's ValidateReport JSON structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidateReport {
    pub spec: String,
    pub mode: String,
    pub phases: Phases,
    pub summary: Summary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Phases {
    pub lint: Option<LintResult>,
    pub generate: Option<Vec<StepResult>>,
    pub compile: Option<Vec<StepResult>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LintResult {
    pub linter: String,
    pub status: String,
    pub log: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepResult {
    pub generator: String,
    pub scope: String,
    pub status: String,
    pub log: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Summary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}

/// Input to the validation pipeline.
pub struct PipelineInput {
    pub config: Config,
    pub custom_defs: Vec<CustomGeneratorDef>,
    pub spec_path: PathBuf,
    pub work_dir: PathBuf,
}

/// Identifies which pipeline phase is running.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Phase {
    Lint,
    Generate { generator: String, scope: String },
    Compile { generator: String, scope: String },
}

/// Events emitted by the pipeline orchestrator.
///
/// Serializes adjacently tagged (`{"type": ..., "data": ...}`) so frontends
/// receive a discriminated union.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum PipelineEvent {
    PhaseStarted(Phase),
    Log {
        phase: Phase,
        line: String,
    },
    PhaseFinished {
        phase: Phase,
        success: bool,
    },
    /// Sent immediately after lint finishes so the TUI can display
    /// lint errors while generate/compile are still running.
    LintCompleted(LintResult),
    Completed(ValidateReport),
    Aborted(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_serializes_internally_tagged() {
        let json = serde_json::to_value(Phase::Lint).unwrap();
        assert_eq!(json, serde_json::json!({"type": "lint"}));

        let json = serde_json::to_value(Phase::Generate {
            generator: "spring".into(),
            scope: "server".into(),
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({"type": "generate", "generator": "spring", "scope": "server"})
        );
    }

    #[test]
    fn pipeline_event_serializes_adjacently_tagged() {
        let json = serde_json::to_value(PipelineEvent::Log {
            phase: Phase::Lint,
            line: "checking".into(),
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "type": "log",
                "data": {"phase": {"type": "lint"}, "line": "checking"}
            })
        );

        let json = serde_json::to_value(PipelineEvent::Aborted("boom".into())).unwrap();
        assert_eq!(json, serde_json::json!({"type": "aborted", "data": "boom"}));
    }

    #[test]
    fn pipeline_event_json_round_trips() {
        let event = PipelineEvent::PhaseFinished {
            phase: Phase::Compile {
                generator: "go".into(),
                scope: "client".into(),
            },
            success: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: PipelineEvent = serde_json::from_str(&json).unwrap();
        match back {
            PipelineEvent::PhaseFinished { phase, success } => {
                assert!(success);
                assert_eq!(
                    phase,
                    Phase::Compile {
                        generator: "go".into(),
                        scope: "client".into(),
                    }
                );
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }
}
