# ETABS API Safety Reference

## Return codes

CSI ETABS API methods commonly return `0` on success and a nonzero integer on failure. Do not ignore this. When answering or coding:

- Say whether the method returns a status code.
- Check the return code immediately after the CSI call.
- In calling code, preserve the documented status behavior instead of ignoring failures.

## Live-model mutation

Only mutate a live ETABS model when the user explicitly asks for it. Mutation includes:

- `Set...`, `Add...`, `Delete...`, `ChangeName...`, `Apply...`.
- `RunAnalysis` or operations that change selected output cases.
- Design preferences, design overwrites, load definitions, units, database table edits, groups, selections, section assignments, or object geometry.

Before mutation, confirm:

- Target model or active ETABS instance.
- Scope: object, group, selected objects, or full model.
- Units and design-code version.
- Whether a backup/export is expected.

## Analysis-result workflow

For results, retrieve both setup and result method cards:

```powershell
cardex get "cAnalysisResultsSetup.SetCaseSelectedForOutput" --index ".cardex\etabs-api" --json
cardex get "cAnalysisResults.FrameForce" --index ".cardex\etabs-api" --json
```

Expected workflow:

1. Deselect or select output cases/combinations intentionally.
2. Run or confirm analysis if needed.
3. Call the specific `cAnalysisResults.*` method.
4. Check return code and array lengths.
5. Interpret result arrays without reordering fields.

## Database table workflow

Database table editing is queued before it is applied.

1. Discover tables: `GetAvailableTables` or `GetAllTables`.
2. Inspect fields: `GetAllFieldsInTable`.
3. Read display data: `GetTableForDisplayArray`, CSV, XML, or file methods.
4. Pull editable data: `GetTableForEditingArray`, CSV string, or CSV file.
5. Preserve the table schema; add/delete rows only when valid.
6. Queue edits with `SetTableForEditing...`.
7. Apply with `ApplyEditedTables` or discard with `CancelTableEditing`.

Treat `ApplyEditedTables` as a model mutation. If the model is locked, only lock-compatible imports may apply.

## Versioned design-code APIs

Do not infer a design code year. Query exact versions when present:

```powershell
cardex search "ACI 318-14 concrete design preference" --index ".cardex\etabs-api" --explain --json
```

If the user says only `ACI 318`, return all available variants and ask which one to use before editing preferences or overwrites.

