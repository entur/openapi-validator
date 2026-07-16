import { useEffect, useRef } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";
import type * as monacoNs from "monaco-editor";
import type { LintError } from "../types";
import type { OpenFile } from "../App";

interface Props {
  file: OpenFile | null;
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

export default function EditorPane({ file, lintErrors, onSave, onBackToSpec }: Props) {
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
    return <main className="editor empty">No file open</main>;
  }

  return (
    <main className="editor">
      <div className="editor-header">
        <span className="filename">
          {file.path}
          {file.readOnly && " (read-only)"}
        </span>
        {onBackToSpec && <button onClick={onBackToSpec}>← Back to spec</button>}
        {!file.readOnly && (
          <button onClick={() => editorRef.current && onSave(editorRef.current.getValue())}>
            Save (⌘S)
          </button>
        )}
      </div>
      <Editor
        path={file.path}
        language={languageFor(file.path)}
        value={file.content}
        theme="vs-dark"
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
