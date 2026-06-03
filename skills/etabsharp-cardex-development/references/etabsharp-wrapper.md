# EtabSharp Wrapper Reference

## Local repo facts

`D:\Work\EtabSharp` contains:

- `src/EtabSharp`: wrapper library.
- `mcp/EtabSharp.Mcp`: MCP server project.
- `test/EtabSharp.Test`: tests.
- `ETABSWrapper.Connect()`: connects to a running ETABS instance.
- `ETABSModel`: exposes wrapper managers for analysis, results, properties, objects, design, tables, and system operations.

EtabSharp supports ETABS v22 and v23 and does not include `ETABSv1.dll`; ETABS must be installed locally.

## Translation workflow

1. Use Cardex to identify the CSI symbol and exact signature.
2. Search EtabSharp for the wrapper method or manager.
3. Prefer the wrapper method when it exists.
4. If the wrapper is missing, implement a wrapper that follows local patterns.
5. Preserve CSI return-code handling and existing exception style.
6. Add or update tests when changing wrapper behavior.

Useful searches:

```powershell
rg -n "FrameForce|JointDispl|SetCaseSelectedForOutput" D:\Work\EtabSharp\src\EtabSharp
rg -n "cAnalysisResults|cDatabaseTables|GetTableForDisplay" D:\Work\EtabSharp\src\EtabSharp
rg -n "EtabsException|ret != 0" D:\Work\EtabSharp\src\EtabSharp
```

## Common mappings

| CSI/Cardex | EtabSharp |
| --- | --- |
| `cAnalyze.*` | `src/EtabSharp/Analyzes/AnalysisManager.cs` |
| `cAnalysisResults.*` | `src/EtabSharp/AnalysisResults/AnalysisResultsManager*.cs` |
| `cAnalysisResultsSetup.*` | `src/EtabSharp/AnalysisResultsSetup/AnalysisResultsSetupManager.cs` |
| `cDatabaseTables.*` | `src/EtabSharp/DatabaseTables/DatabaseTableManager.cs` |
| `cPropFrame.*` | `src/EtabSharp/Properties/Frames/FramePropertyManager.cs` |
| `cFrameObj.*` | `src/EtabSharp/Elements/FrameObj/FrameObjectManager*.cs` |
| `cPointObj.*` | `src/EtabSharp/Elements/PointObj/PointObjectManager*.cs` |
| `cDesignConcrete`, `cDCo*` | `src/EtabSharp/Design/Concrete/**` |
| `cDesignSteel`, `cDSt*` | `src/EtabSharp/Design/Steel/**` |

## Wrapper style checklist

When adding a wrapper method:

- Match existing manager/interface naming.
- Keep CSI parameter order visible in comments or implementation.
- Convert parallel ETABS arrays into typed result models carefully.
- Check `ret` and throw `EtabsException` where neighboring methods do so.
- Do not silently change units, selections, or output cases.
- For versioned design APIs, keep code-specific classes separate, such as `ACI318_14` or `AISC360_16`.

## Database table note

Use Cardex to confirm database-table edit/apply semantics before wrapping or exposing helper methods. `SetTableForEditing...` queues edits; `ApplyEditedTables` mutates the model.

## MCP note

EtabSharp has an MCP project, but Cardex remains CLI-first for documentation retrieval. Use Cardex CLI for ETABS API facts; use EtabSharp MCP only when the user is specifically working with that project/tooling.
