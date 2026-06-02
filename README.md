# Cardex

Cardex is a Rust-first, local/offline documentation retrieval engine for engineering software agents.

The first target corpus is CSI ETABS API documentation extracted from a CHM file. Cardex turns the decompiled HTML topics into compact, searchable API cards so agents can query exact signatures and related docs without loading full manuals into context.

## Current Scope

- Parse nested `.hhc` table-of-contents files to recover symbols such as `cAnalysisResults.FrameForce`.
- Read decompiled `.htm` topic pages.
- Generate structured artifacts:
  - `pages.jsonl`
  - `docgraph.json`
  - `manifest.json`
  - `tantivy/`
- Provide a CLI over the generated index:
  - `cardex build`
  - `cardex search`
  - `cardex get`
  - `cardex members`
  - `cardex related`
- Keep generated/proprietary docs and indexes out of Git.

MCP is intentionally not implemented yet.

## Repository Layout

```text
BUILD_REPORT.md  # root-level ETABS smoke build notes
crates/
  cardex-core/   # ingest, API cards, DocGraph, artifact loading, BM25 search
  cardex-cli/    # clap CLI wrapper around cardex-core
docs/
  README.md
  superpowers/
    build-report-etabs-api.md
    plans/       # implementation notes and task tracking
```

For more detail, see [BUILD_REPORT.md](BUILD_REPORT.md) and [docs/superpowers/build-report-etabs-api.md](docs/superpowers/build-report-etabs-api.md).

## Build And Test

```powershell
cargo fmt --all
cargo test --workspace
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Build A Local Index

Use a decompiled CHM folder that contains one `.hhc` file and the linked `.htm` pages:

```powershell
cargo run -p cardex-cli -- build `
  --source "D:\path\to\decompiled-etabs-api" `
  --out ".cardex\etabs-api" `
  --corpus etabs-api `
  --json
```

Generated artifacts are ignored by Git.

### Local ETABS Smoke Build

This repo was smoke-tested on this machine against the installed ETABS 23 API CHM after decompiling it locally. The proprietary source and generated artifacts are not committed.

```json
{
  "corpus": "etabs-api",
  "pages": 1798,
  "hhc_entries": 1801,
  "output_dir": ".cardex\\etabs-api"
}
```

Representative smoke checks:

- `search "frame force"` ranked `cAnalysisResults.FrameForce` first.
- `get "cAnalysisResults.FrameForce"` returned compact return text, remarks, related docs, and 16 parameters.
- `members "cAnalysisResults"` returned 37 callable members.
- `related "cAnalysisResults.FrameForce"` returned focused See Also targets such as `ETABSv1` and `cAnalysisResults`.

## Query The Index

```powershell
cargo run -p cardex-cli -- search "frame force" --index ".cardex\etabs-api" --json
cargo run -p cardex-cli -- get "cAnalysisResults.FrameForce" --index ".cardex\etabs-api" --json
cargo run -p cardex-cli -- members "cAnalysisResults" --index ".cardex\etabs-api" --json
cargo run -p cardex-cli -- related "cAnalysisResults.FrameForce" --index ".cardex\etabs-api" --json
```

## Agent Retrieval Policy

- L1: `search` returns compact ranked hits.
- L2: `get` returns one structured API card.
- L3: use raw/full page text only when the compact card is ambiguous.

Agents should search Cardex first, fetch only the target API card, check CSI return codes, and avoid mutating live ETABS models unless explicitly requested.
