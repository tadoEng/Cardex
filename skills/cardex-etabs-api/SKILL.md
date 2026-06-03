---
name: cardex-etabs-api
description: Use this skill whenever the user asks about CSI ETABS API behavior, ETABS API symbols, design-code APIs, database tables, analysis results, return codes, or how to retrieve ETABS documentation through Cardex. This skill tells agents to search local Cardex first, fetch compact API cards instead of full manuals, use members/related graph navigation, preserve design-code version safety, check CSI return codes, and avoid mutating live ETABS models unless explicitly requested.
---

# Cardex ETABS API Retrieval

## Core rule

Use Cardex before answering ETABS API questions. Cardex is the local documentation memory; this skill is the retrieval and safety policy.

Prefer this flow:

1. Search Cardex for the user intent.
2. Fetch only the target API card.
3. Use `members` or `related` to navigate interfaces.
4. Answer from ETABS API facts, not memory guesses.
5. Check CSI return codes.
6. Avoid live-model mutation unless the user asked for it.

This skill is pure Cardex + CSI ETABS API. Wrapper-specific design belongs in a separate skill or project workflow.

## Find the index and CLI

Default index locations to try, in order:

```powershell
.cardex\etabs-api
D:\Work\Cardex\.cardex\etabs-api
```

Default CLI commands:

```powershell
cardex search "frame force" --index ".cardex\etabs-api" --json
cardex get "cAnalysisResults.FrameForce" --index ".cardex\etabs-api" --json
cardex members "cAnalysisResults" --index ".cardex\etabs-api" --json
cardex related "cAnalysisResults.FrameForce" --index ".cardex\etabs-api" --json
```

If `cardex` is not on PATH, run from the Cardex repo:

```powershell
cargo run -p cardex-cli -- search "frame force" --index ".cardex\etabs-api" --json
```

On this machine, the debug binary may also exist at:

```powershell
D:\Work\Cardex\.codex-target\debug\cardex.exe
```

Read `references/cardex-cli.md` when you need command examples, JSON shapes, or query tactics.

## Retrieval levels

Use the smallest useful retrieval level:

- L1: `search` gives compact ranked hits.
- L2: `get` gives one structured API card.
- L2 graph: `members` and `related` navigate interfaces and neighbors.
- L3: raw/full page text only when the compact card is ambiguous.

For broad or suspicious natural-language queries, add explain mode:

```powershell
cardex search "ACI 318-14 concrete frame design requirement" --index ".cardex\etabs-api" --explain --json
```

Use explain output to see normalized terms, version scope, fallback stage, seed symbols, and graph promotions.

## Version safety

Never guess design-code years.

- Explicit `ACI 318-14`, `ACI318-14`, or `ACI 318_14` should stay scoped to `ACI318_14` and `_14` symbols.
- Bare `ACI 318` should return available variants, not a preferred/latest year.
- Apply the same caution to AISC, Eurocode, or any year/versioned design API.

If the user's query omits a version but the operation depends on one, return the grouped variants and ask the user to choose before changing settings or writing exact code.

## ETABS safety policy

Treat ETABS as a live engineering model unless the user says otherwise.

Safe by default:

- Search/get docs.
- Read model info.
- Read analysis results after confirming output cases are selected.
- Inspect available database tables or fields.

Requires explicit user intent:

- `Set...`, `Delete...`, `Add...`, `ChangeName...`, `RunAnalysis`, `ApplyEditedTables`.
- Changing design preferences, design overwrites, load cases, units, selections, or database-table edits.
- Any operation that mutates a live ETABS model or runs analysis.

Always preserve the CSI return-code rule: CSI API methods usually return `0` for success and nonzero for failure.

Read `references/etabs-safety.md` before writing or recommending anything that mutates ETABS or applies database table edits.

## Good answer shape

For ETABS API answers, include:

- The Cardex query used, if relevant.
- The target symbol or page title.
- The practical API workflow.
- Return-code or mutation caveats.
- Any unresolved ambiguity such as design-code year, selected result case, object scope, or units.

Keep proprietary generated docs out of commits. Do not commit CHM files, decompiled HTML, generated Markdown, `.cardex` artifacts, or full Tantivy indexes.
