import { useState } from "react";
import { Modal } from "@entur/modal";
import { ButtonGroup, PrimaryButton, SecondaryButton } from "@entur/button";
import { TextField } from "@entur/form";

interface Props {
  open: boolean;
  onDismiss: () => void;
  /** Fetches the spec into the workspace; rejects with a message on failure. */
  onFetch: (url: string) => Promise<void>;
}

export default function UrlModal({ open, onDismiss, onFetch }: Props) {
  const [url, setUrl] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const trimmed = url.trim();
  const valid = trimmed.startsWith("http://") || trimmed.startsWith("https://");

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!valid || loading) return;
    setLoading(true);
    setError(null);
    try {
      await onFetch(trimmed);
      setUrl("");
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal open={open} onDismiss={onDismiss} title="Open spec from URL" size="medium">
      <form onSubmit={submit}>
        <TextField
          label="Spec URL"
          placeholder="https://example.com/openapi.yaml"
          value={url}
          onChange={(e) => {
            setUrl(e.target.value);
            setError(null);
          }}
          variant={error ? "negative" : undefined}
          feedback={error ?? "The spec is downloaded into the workspace as .oav/fetched-spec"}
        />
        <ButtonGroup className="url-modal__actions">
          <PrimaryButton type="submit" loading={loading} disabled={!valid}>
            Fetch spec
          </PrimaryButton>
          <SecondaryButton type="button" onClick={onDismiss} disabled={loading}>
            Cancel
          </SecondaryButton>
        </ButtonGroup>
      </form>
    </Modal>
  );
}
