---
name: etabsharp-cardex-development
description: Use this skill when designing, implementing, reviewing, or testing EtabSharp wrapper code for CSI ETABS API features. This skill tells agents to use Cardex as the source of ETABS API truth, retrieve exact symbols/signatures/return behavior first, then map the API to EtabSharp managers, interfaces, models, return-code handling, and tests without mixing proprietary Cardex indexes into the wrapper repo.
---

# EtabSharp Development With Cardex

## Core rule

Use Cardex to understand the CSI ETABS API before designing or changing EtabSharp. Cardex answers "what does ETABS expose?" EtabSharp answers "how should this be wrapped in .NET?"

Do not put EtabSharp assumptions into Cardex. Keep the direction one-way:

```text
Cardex ETABS API cards -> EtabSharp wrapper design/code/tests
```

## Workflow

1. Use the `cardex-etabs-api` retrieval policy to find exact ETABS API cards.
2. Fetch the target CSI symbol with `cardex get`.
3. Use `members` for interface coverage and `related` for adjacent docs.
4. Search EtabSharp for an existing manager, interface, model, or method.
5. Prefer existing EtabSharp patterns over raw ETABS calls in user code.
6. Preserve CSI return-code behavior through EtabSharp exceptions or typed results.
7. Add focused tests for wrapper mapping, array conversion, and mutation safety.

Read `references/etabsharp-wrapper.md` for local repo layout and mapping examples.

## Cardex queries for wrapper work

Examples:

```powershell
cardex get "cAnalysisResults.FrameForce" --index "D:\Work\Cardex\.cardex\etabs-api" --json
cardex members "cAnalysisResults" --index "D:\Work\Cardex\.cardex\etabs-api" --json
cardex get "cDatabaseTables.ApplyEditedTables" --index "D:\Work\Cardex\.cardex\etabs-api" --json
cardex search "ACI 318-14 concrete design preference" --index "D:\Work\Cardex\.cardex\etabs-api" --explain --json
```

If a design-code query is versioned, keep that version in the EtabSharp type or namespace. Do not silently map a bare code family to a latest year.

## Wrapper implementation policy

When adding or changing EtabSharp:

- Keep the ETABS API symbol visible in XML comments or implementation comments when helpful.
- Match existing manager/interface organization.
- Convert ETABS parallel arrays into typed models without reordering fields.
- Check nonzero CSI return codes.
- Treat `Set...`, `Add...`, `Delete...`, `RunAnalysis`, and `ApplyEditedTables` as mutation paths.
- Do not change units, result selection, active object selection, or design preferences as a hidden side effect.

## Completion evidence

For EtabSharp changes, report:

- Cardex symbols queried.
- EtabSharp files changed.
- Return-code handling strategy.
- Tests or checks run.
- Any live ETABS action skipped or requiring user approval.
