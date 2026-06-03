# Cardex CLI Reference

## Commands

Use JSON for agent workflows:

```powershell
cardex search "<query>" --index ".cardex\etabs-api" --limit 10 --json
cardex search "<query>" --index ".cardex\etabs-api" --limit 10 --explain --json
cardex get "<symbol-or-page-id>" --index ".cardex\etabs-api" --json
cardex members "<interface>" --index ".cardex\etabs-api" --json
cardex related "<symbol>" --index ".cardex\etabs-api" --json
```

If the installed command is unavailable, use:

```powershell
cargo run -p cardex-cli -- search "<query>" --index ".cardex\etabs-api" --json
```

or the local debug binary when present:

```powershell
D:\Work\Cardex\.codex-target\debug\cardex.exe search "<query>" --index "D:\Work\Cardex\.cardex\etabs-api" --json
```

## Query tactics

Use natural language first for discovery, then exact symbols for retrieval.

Examples:

```powershell
cardex search "joint displacement results output" --index ".cardex\etabs-api" --json
cardex get "cAnalysisResults.JointDispl" --index ".cardex\etabs-api" --json
cardex members "cAnalysisResults" --index ".cardex\etabs-api" --json
```

For interface workflows:

```powershell
cardex search "database table editing display workflow" --index ".cardex\etabs-api" --json
cardex members "cDatabaseTables" --index ".cardex\etabs-api" --json
cardex get "cDatabaseTables.ApplyEditedTables" --index ".cardex\etabs-api" --json
```

For design-code APIs:

```powershell
cardex search "ACI 318-14 concrete frame design requirement" --index ".cardex\etabs-api" --explain --json
cardex get "cDCoACI318_14.GetPreference" --index ".cardex\etabs-api" --json
cardex members "cDCoACI318_14" --index ".cardex\etabs-api" --json
```

## Search interpretation

- Prefer exact symbol hits over broad overview pages when writing code.
- Use overview pages for conceptual workflow questions.
- Use `members` when the user asks "how many", "what methods", "what outputs", or "what does this interface support".
- Use `related` to follow See Also links without loading unrelated manual sections.
- Use `--explain --json` when results seem weak, versioned, or graph-promoted.

## Common ETABS symbol families

| User intent | Start with |
| --- | --- |
| Run analysis | `cAnalyze` |
| Select result output cases | `cAnalysisResultsSetup` |
| Analysis results | `cAnalysisResults` |
| Database tables | `cDatabaseTables` |
| Frame sections | `cPropFrame` |
| Frame objects | `cFrameObj` |
| Point/joint objects | `cPointObj` |
| Concrete design | `cDesignConcrete`, `cDCo*` |
| Steel design | `cDesignSteel`, `cDSt*` |

## Failure handling

If the index does not exist, do not fetch proprietary docs from the internet. Ask for or build from a local decompiled CHM source. Generated Cardex artifacts are local and should remain ignored by Git.
