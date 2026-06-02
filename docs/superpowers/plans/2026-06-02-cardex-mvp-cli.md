# Cardex MVP CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first Cardex CLI that ingests decompiled ETABS CHM HTML into structured API cards and exposes build, search, get, and members commands.

**Architecture:** `cardex-core` owns HHC parsing, HTML extraction, API card generation, DocGraph creation, artifact IO, and BM25 search. `cardex-cli` is a thin `clap` wrapper that calls core APIs and emits compact human output by default or JSON for agents.

**Tech Stack:** Rust workspace, `scraper`, `tantivy`, `serde`, `serde_json`, `thiserror`, `anyhow`, `clap`, `tracing`.

---

### Task 1: Workspace Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `crates/cardex-core/Cargo.toml`
- Create: `crates/cardex-core/src/lib.rs`
- Create: `crates/cardex-cli/Cargo.toml`
- Create: `crates/cardex-cli/src/main.rs`

- [ ] Create a Cargo workspace with `cardex-core` and `cardex-cli`.
- [ ] Ignore generated docs/index artifacts including extracted HTML, markdown, `pages.jsonl`, `docgraph.json`, `manifest.json`, and Tantivy index directories.
- [ ] Run `cargo check --workspace` and expect an empty workspace to compile before behavior tests are added.

### Task 2: HHC TOC Parsing

**Files:**
- Test: `crates/cardex-core/tests/hhc_parse.rs`
- Modify: `crates/cardex-core/src/lib.rs`
- Create: `crates/cardex-core/src/hhc.rs`
- Create: `crates/cardex-core/src/model.rs`

- [ ] Write a failing test with nested `.hhc` HTML where `cAnalysisResults` contains `FrameForce Method`.
- [ ] Verify the test fails because `parse_hhc` and the TOC model do not exist.
- [ ] Implement `parse_hhc` using `scraper` to read nested `<ul>/<li>/<object>` entries, preserving title, local path, depth, ancestors, and fully-qualified symbols.
- [ ] Run the HHC test and expect `cAnalysisResults.FrameForce` for the method page.

### Task 3: Page Extraction and API Cards

**Files:**
- Test: `crates/cardex-core/tests/cards.rs`
- Modify: `crates/cardex-core/src/lib.rs`
- Create: `crates/cardex-core/src/cards.rs`
- Modify: `crates/cardex-core/src/model.rs`

- [ ] Write failing tests for extracting title, C# signature, VB signature, parameters, return-code sentence, remarks, and related links from representative HTML.
- [ ] Verify the tests fail for missing extraction APIs.
- [ ] Implement a conservative `build_card_from_html` that prefers headings, code/pre blocks, tables, and see-also links, while keeping `raw_text` as searchable fallback.
- [ ] Run the card tests and expect compact `ApiCard` structs with stable JSON serialization.

### Task 4: Build Artifacts and DocGraph

**Files:**
- Test: `crates/cardex-core/tests/build_index.rs`
- Modify: `crates/cardex-core/src/lib.rs`
- Create: `crates/cardex-core/src/build.rs`
- Create: `crates/cardex-core/src/store.rs`
- Create: `crates/cardex-core/src/search.rs`

- [ ] Write failing tests that create a temporary corpus with `.hhc` plus two `.htm` pages and call `build_corpus`.
- [ ] Verify the tests fail for missing build APIs.
- [ ] Implement artifact writing for `pages.jsonl`, `docgraph.json`, `manifest.json`, and a Tantivy index directory.
- [ ] Implement artifact loading and search by query/symbol/title/interface.
- [ ] Run build/search tests and expect L1 hits, L2 card retrieval, and interface members from generated artifacts.

### Task 5: CLI Surface

**Files:**
- Test: `crates/cardex-cli/tests/cli.rs`
- Modify: `crates/cardex-cli/src/main.rs`

- [ ] Write failing CLI tests with `assert_cmd` for `build`, `search --json`, `get --json`, and `members --json`.
- [ ] Verify the CLI tests fail before command implementation.
- [ ] Implement `cardex build --source <dir> --out <dir> --corpus etabs-api`, `cardex search <query> --index <dir> --limit N --json`, `cardex get <symbol> --index <dir> --json`, and `cardex members <interface> --index <dir> --json`.
- [ ] Run CLI tests and expect JSON output useful to software agents.

### Task 6: Final Verification

**Files:**
- All workspace files touched above.

- [ ] Run `cargo fmt --all`.
- [ ] Run `cargo test --workspace`.
- [ ] Run `cargo check --workspace`.
- [ ] Confirm no proprietary CSI documentation or generated full indexes are committed.
