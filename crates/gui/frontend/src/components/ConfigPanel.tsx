import { Checkbox, Fieldset, SegmentedControl, SegmentedChoice, Switch } from "@entur/form";
import { NativeDropdown } from "@entur/dropdown";
import { Heading5 } from "@entur/typography";
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
    <Fieldset label={`${scope === "server" ? "Server" : "Client"} generators`}>
      <div className="sidebar__generators">
        {names(scope).map((name) => (
          <Checkbox
            key={`${scope}/${name}`}
            checked={config[
              scope === "server" ? "server_generators" : "client_generators"
            ].includes(name)}
            onChange={() => toggle(scope, name)}
          >
            {name}
            {catalog.custom.some((c) => c.name === name && c.scope === scope) &&
              " (custom)"}
          </Checkbox>
        ))}
      </div>
    </Fieldset>
  );

  return (
    <aside className="sidebar" aria-label="Pipeline configuration">
      <section className="sidebar__section">
        <Heading5 as="h2">Pipeline</Heading5>
        <SegmentedControl
          label="Mode"
          value={config.mode}
          onChange={(value) => value && onChange({ ...config, mode: value as Mode })}
        >
          <SegmentedChoice value="server">Server</SegmentedChoice>
          <SegmentedChoice value="client">Client</SegmentedChoice>
          <SegmentedChoice value="both">Both</SegmentedChoice>
        </SegmentedControl>
        <NativeDropdown
          label="Linter"
          items={[
            { value: "spectral", label: "Spectral" },
            { value: "redocly", label: "Redocly" },
            { value: "none", label: "None" },
          ]}
          selectedItem={{ value: config.linter, label: config.linter }}
          onChange={(item) =>
            item && onChange({ ...config, linter: item.value as Linter })
          }
        />
        <div className="sidebar__switches">
          <Switch
            checked={config.lint}
            onChange={(e) => onChange({ ...config, lint: e.target.checked })}
          >
            Lint
          </Switch>
          <Switch
            checked={config.generate}
            onChange={(e) => onChange({ ...config, generate: e.target.checked })}
          >
            Generate
          </Switch>
          <Switch
            checked={config.compile}
            onChange={(e) => onChange({ ...config, compile: e.target.checked })}
          >
            Compile
          </Switch>
        </div>
      </section>
      {(config.mode === "server" || config.mode === "both") && (
        <section className="sidebar__section">{generatorList("server")}</section>
      )}
      {(config.mode === "client" || config.mode === "both") && (
        <section className="sidebar__section">{generatorList("client")}</section>
      )}
    </aside>
  );
}
