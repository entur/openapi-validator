import type { Catalog, Config, Linter, Mode } from "../types";

interface Props {
  config: Config;
  catalog: Catalog;
  onChange: (next: Config) => void;
}

export default function ConfigPanel({ config, catalog, onChange }: Props) {
  const names = (scope: "server" | "client") => [
    ...catalog.builtin.filter((g) => g.scope === scope).map((g) => g.name),
    ...catalog.custom.filter((g) => g.scope === scope).map((g) => g.name),
  ];

  const toggle = (scope: "server" | "client", name: string) => {
    const key = scope === "server" ? "server_generators" : "client_generators";
    const current = config[key];
    const next = current.includes(name)
      ? current.filter((n) => n !== name)
      : [...current, name];
    onChange({ ...config, [key]: next });
  };

  const generatorList = (scope: "server" | "client") => (
    <ul className="generator-list">
      {names(scope).map((name) => (
        <li key={`${scope}/${name}`}>
          <label>
            <input
              type="checkbox"
              checked={config[
                scope === "server" ? "server_generators" : "client_generators"
              ].includes(name)}
              onChange={() => toggle(scope, name)}
            />
            {name}
            {catalog.custom.some((c) => c.name === name && c.scope === scope) && (
              <em> (custom)</em>
            )}
          </label>
        </li>
      ))}
    </ul>
  );

  return (
    <aside className="sidebar">
      <section>
        <h3>Pipeline</h3>
        <label>
          Mode{" "}
          <select
            value={config.mode}
            onChange={(e) => onChange({ ...config, mode: e.target.value as Mode })}
          >
            <option value="server">server</option>
            <option value="client">client</option>
            <option value="both">both</option>
          </select>
        </label>
        <label>
          Linter{" "}
          <select
            value={config.linter}
            onChange={(e) => onChange({ ...config, linter: e.target.value as Linter })}
          >
            <option value="spectral">spectral</option>
            <option value="redocly">redocly</option>
            <option value="none">none</option>
          </select>
        </label>
        <label>
          <input
            type="checkbox"
            checked={config.lint}
            onChange={(e) => onChange({ ...config, lint: e.target.checked })}
          />
          Lint
        </label>
        <label>
          <input
            type="checkbox"
            checked={config.generate}
            onChange={(e) => onChange({ ...config, generate: e.target.checked })}
          />
          Generate
        </label>
        <label>
          <input
            type="checkbox"
            checked={config.compile}
            onChange={(e) => onChange({ ...config, compile: e.target.checked })}
          />
          Compile
        </label>
      </section>
      {(config.mode === "server" || config.mode === "both") && (
        <section>
          <h3>Server generators</h3>
          {generatorList("server")}
        </section>
      )}
      {(config.mode === "client" || config.mode === "both") && (
        <section>
          <h3>Client generators</h3>
          {generatorList("client")}
        </section>
      )}
    </aside>
  );
}
