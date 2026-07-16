import { Modal } from "@entur/modal";
import { ButtonGroup, PrimaryButton, SecondaryButton } from "@entur/button";
import { Paragraph, PreformattedText } from "@entur/typography";
import type { FixProposal } from "../types";

interface Props {
  proposal: FixProposal | null;
  onApply: () => void;
  onDismiss: () => void;
}

export default function FixModal({ proposal, onApply, onDismiss }: Props) {
  if (!proposal) return null;
  const preview = [
    ...proposal.context_before.map((l) => `  ${l}`),
    ...proposal.inserted.map((l) => `+ ${l}`),
    ...proposal.context_after.map((l) => `  ${l}`),
  ].join("\n");

  return (
    <Modal
      open
      onDismiss={onDismiss}
      title={`Fix: ${proposal.rule}`}
      size="medium"
    >
      <Paragraph>{proposal.description}</Paragraph>
      <PreformattedText className="fix-modal__preview">{preview}</PreformattedText>
      <ButtonGroup>
        <PrimaryButton onClick={onApply}>Apply fix</PrimaryButton>
        <SecondaryButton onClick={onDismiss}>Cancel</SecondaryButton>
      </ButtonGroup>
    </Modal>
  );
}
