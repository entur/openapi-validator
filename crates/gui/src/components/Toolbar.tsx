import { Logo } from "@entur/menu";
import { Contrast, StatusBadge } from "@entur/layout";
import { PrimaryButton, NegativeButton, SecondaryButton, TertiaryButton } from "@entur/button";
import { Dropdown } from "@entur/dropdown";
import { FolderIcon, ExternalIcon } from "@entur/icons";
import { useMediaQuery } from "../hooks";

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

/**
 * Entur logo that adapts to window width:
 * narrow → logo only, medium → "Entur OAV", wide → full product name.
 * Breakpoints follow the Entur tokens (--breakpoints-large / -extra-large).
 */
function ResponsiveLogo() {
  const medium = useMediaQuery("(min-width: 50rem)");
  const large = useMediaQuery("(min-width: 75rem)");
  const productName = large ? "OpenAPI Validator" : medium ? "OAV" : undefined;
  return <Logo productName={productName} size="small" className="toolbar__logo" />;
}

export default function Toolbar(props: Props) {
  const specItems = props.specs.map((s) => ({ value: s, label: s }));
  if (props.specPath && !props.specs.includes(props.specPath)) {
    specItems.push({ value: props.specPath, label: props.specPath });
  }
  const folderName = props.root?.split("/").pop();

  return (
    <Contrast as="header" className="toolbar">
      <ResponsiveLogo />
      <SecondaryButton onClick={props.onOpenFolder} title={props.root ?? undefined}>
        <FolderIcon aria-hidden="true" /> {folderName ?? "Open folder…"}
      </SecondaryButton>
      <TertiaryButton onClick={props.onOpenUrl} disabled={!props.root}>
        <ExternalIcon aria-hidden="true" /> Open URL…
      </TertiaryButton>
      <div className="toolbar__spec">
        <Dropdown
          label="Spec"
          items={specItems}
          selectedItem={
            props.specPath ? { value: props.specPath, label: props.specPath } : null
          }
          onChange={(item) => {
            if (item) props.onSelectSpec(item.value);
          }}
          disabled={specItems.length === 0}
          labelClearSelectedItem="Clear selection"
        />
      </div>
      <div className="toolbar__spacer" />
      <StatusBadge variant={props.dockerError ? "negative" : "success"}>
        <span title={props.dockerError ?? "Docker available"}>Docker</span>
      </StatusBadge>
      {props.running ? (
        <NegativeButton onClick={props.onCancel}>
          Cancel
        </NegativeButton>
      ) : (
        <PrimaryButton
          onClick={props.onRun}
          disabled={!props.specPath || props.dockerError !== null}
        >
          Validate
        </PrimaryButton>
      )}
    </Contrast>
  );
}
