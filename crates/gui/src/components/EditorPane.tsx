import { useEffect, useRef } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";
import type * as monacoNs from "monaco-editor";
import { SecondaryButton } from "@entur/button";
import { BackArrowIcon, SaveIcon } from "@entur/icons";
import { Label } from "@entur/typography";
import { Tag } from "@entur/layout";
import type { LintError } from "../types";
import type { OpenFile } from "../App";

interface Props {
  file: OpenFile | null;
  colorMode: "light" | "dark";
  lintErrors: LintError[];
  onSave: (content: string) => void;
  onBackToSpec?: () => void;
}

function languageFor(path: string): string {
  if (path.endsWith(".json")) return "json";
  if (path.endsWith(".yaml") || path.endsWith(".yml")) return "yaml";
  if (path.endsWith(".ts")) return "typescript";
  if (path.endsWith(".java") || path.endsWith(".kt")) return "java";
  if (path.endsWith(".go")) return "go";
  if (path.endsWith(".py")) return "python";
  if (path.endsWith(".cs")) return "csharp";
  return "plaintext";
}

const severityMap: Record<LintError["severity"], monacoNs.MarkerSeverity | 8> = {
  error: 8, // MarkerSeverity.Error
  warning: 4,
  info: 2,
  hint: 1,
};

export default function EditorPane({
  file,
  colorMode,
  lintErrors,
  onSave,
  onBackToSpec,
}: Props) {
  const editorRef = useRef<monacoNs.editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<typeof monacoNs | null>(null);
  const onSaveRef = useRef(onSave);
  onSaveRef.current = onSave;

  const handleMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      onSaveRef.current(editor.getValue());
    });
  };

  // Push lint errors into monaco markers whenever they change.
  useEffect(() => {
    const editor = editorRef.current;
    const monaco = monacoRef.current;
    if (!editor || !monaco) return;
    const model = editor.getModel();
    if (!model) return;
    monaco.editor.setModelMarkers(
      model,
      "oav-lint",
      lintErrors.map((e) => ({
        severity: severityMap[e.severity],
        message: `${e.rule}: ${e.message}`,
        startLineNumber: e.line,
        startColumn: e.col + 1,
        endLineNumber: e.line,
        endColumn: model.getLineMaxColumn(Math.min(e.line, model.getLineCount())),
      })),
    );
  }, [lintErrors, file?.path]);

  if (!file) {
    return (
      <main id="main-content" className="editor editor--empty">
        <Label>Open a folder to start editing a spec</Label>
      </main>
    );
  }

  return (
    <main id="main-content" className="editor">
      <div className="editor__header">
        <span className="editor__filename" title={file.path}>
          {file.path}
        </span>
        {file.readOnly && <Tag>read-only</Tag>}
        {onBackToSpec && (
          <SecondaryButton onClick={onBackToSpec}>
            <BackArrowIcon aria-hidden="true" /> Back to spec
          </SecondaryButton>
        )}
        {!file.readOnly && (
          <SecondaryButton
            onClick={() => editorRef.current && onSave(editorRef.current.getValue())}
          >
            <SaveIcon aria-hidden="true" /> Save
          </SecondaryButton>
        )}
      </div>
      <Editor
        path={file.path}
        language={languageFor(file.path)}
        value={file.content}
        theme={colorMode === "dark" ? "vs-dark" : "vs"}
        options={{
          readOnly: file.readOnly,
          minimap: { enabled: false },
          fontSize: 13,
          scrollBeyondLastLine: false,
        }}
        onMount={handleMount}
      />
    </main>
  );
}
