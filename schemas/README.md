# Kali schema documents

These JSON Schema documents describe the schema-v1 machine-readable surfaces used by the Kali CLI
and related artifacts. They are intentionally narrow and deterministic so downstream tooling can
validate envelopes, diagnostics, manifests, lockfiles, policies, and artifact metadata without
scraping prose.

Reserved shapes for later commands live here as well so names stay stable before the commands ship.
