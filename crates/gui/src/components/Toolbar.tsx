interface Props {
  root: string | null;
  specs: string[];
  specPath: string | null;
  running: boolean;
  dockerError: string | null;
  onOpenFolder: () => void;
  onOpenUrl: () => void;
  onSelectSpec: (path: string) => void;
  onRun: () => void;
  onCancel: () => void;
}

export default function Toolbar(props: Props) {
  const folderLabel = props.root ? props.root.split("/").pop() : "Open Folder…";
  return (
    <header className="toolbar">
      <button onClick={props.onOpenFolder} title={props.root ?? undefined}>
        📁 {folderLabel}
      </button>
      <button onClick={props.onOpenUrl} disabled={!props.root}>
        🌐 Open URL…
      </button>
      <select
        value={props.specPath ?? ""}
        onChange={(e) => props.onSelectSpec(e.target.value)}
        disabled={props.specs.length === 0}
      >
        {props.specs.length === 0 && <option value="">no specs found</option>}
        {props.specPath && !props.specs.includes(props.specPath) && (
          <option value={props.specPath}>{props.specPath}</option>
        )}
        {props.specs.map((s) => (
          <option key={s} value={s}>
            {s}
          </option>
        ))}
      </select>
      <div className="spacer" />
      <span
        className={`docker-dot ${props.dockerError ? "bad" : "good"}`}
        title={props.dockerError ?? "Docker available"}
      >
        ● Docker
      </span>
      {props.running ? (
        <button className="cancel" onClick={props.onCancel}>
          ■ Cancel
        </button>
      ) : (
        <button
          className="run"
          onClick={props.onRun}
          disabled={!props.specPath || props.dockerError !== null}
        >
          ▶ Validate
        </button>
      )}
    </header>
  );
}
