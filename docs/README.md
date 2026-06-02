# Cardex - API Documentation Indexing System

A Rust-based system for building searchable indices from API documentation HTML archives (CHM-derived content). Demonstrating with the ETABS structural engineering API.

## 📋 Documentation Structure

```
docs/
├── README.md (this file)
└── superpowers/
    ├── README.md                    # Detailed project overview
    ├── build-report-etabs-api.md   # Build results and API inventory
    └── plans/
        └── 2026-06-02-cardex-mvp-cli.md  # Original MVP specification
```

## 🎯 Quick Links

| Document | Purpose |
|----------|---------|
| [Project Overview](superpowers/README.md) | Architecture, features, and development guide |
| [Build Report](superpowers/build-report-etabs-api.md) | Results from indexing ETABS API docs |
| [MVP Plan](superpowers/plans/2026-06-02-cardex-mvp-cli.md) | Original specification and implementation tasks |

## 🚀 Getting Started

### Build the ETABS API Index
```bash
cd c:\Work\Code\Cardex
cargo run --bin cardex -- build \
  --source "C:\Work\Code\etabApi\CSI_API_ETABS_v1_html" \
  --out ".cardex" \
  --corpus etabs-api
```

### Search the Index
```bash
# Find API documentation
cargo run --bin cardex -- search "cPointElm" --index .cardex

# List interface methods
cargo run --bin cardex -- members "cPointElm" --index .cardex --json

# Follow See Also relationships
cargo run --bin cardex -- related "cAnalysisResults.FrameForce" --index .cardex --json
```

## 📊 Current Status

✅ **MVP Complete**
- HHC parser: Working
- HTML extractor: Working
- Search index: Working (Tantivy BM25)
- CLI tool: Working

📈 **Build Results**
- Total pages indexed: **1,776**
- Main interfaces: **20+**
- Search performance: **<100ms**
- Index size: **~500MB**
- Current ETABS 23 local smoke build: **1,798 pages / 1,801 HHC entries**

## 🏗️ Architecture

**Two-Crate Rust Workspace:**

```
cardex-core/
└── Core indexing library
    ├── HHC TOC parser
    ├── HTML card extractor
    ├── Tantivy search indexer
    └── JSON artifact writer

cardex-cli/
└── Command-line interface
    ├── build   - Create index
    ├── search  - Full-text search
    ├── get     - Retrieve card
    ├── members - List interface methods
    └── related - Follow See Also relationships
```

## 🔍 API Overview

The indexed ETABS API provides 20+ main interfaces:

| Interface | Purpose |
|-----------|---------|
| **cOAPI** | API entry point |
| **cFile** | Model I/O |
| **cPointElm** | Point elements |
| **cAreaElm** | Area elements |
| **cAnalyze** | Analysis execution |
| **cDesignResults** | Results extraction |
| **cStory** | Story management |
| **cView** | 3D visualization |
| + 12 more... | ... |

See [Build Report](superpowers/build-report-etabs-api.md) for complete list.

## 📝 Example Workflows

### Workflow 1: Search for API functionality
```bash
cargo run --bin cardex -- search "Load" --index .cardex --limit 10
cargo run --bin cardex -- search "Constraint" --index .cardex --limit 10
cargo run --bin cardex -- search "Analysis" --index .cardex --limit 5
```

### Workflow 2: Explore an interface
```bash
cargo run --bin cardex -- members "cOAPI" --index .cardex --json
```

Output:
```json
[
  "cOAPI.AttachToInstance",
  "cOAPI.CreateSAPIObject",
  "cOAPI.SetVerbosity",
  ...
]
```

### Workflow 3: Follow related documentation
```bash
cargo run --bin cardex -- related "cAnalysisResults.FrameForce" --index .cardex --json
```

### Workflow 4: Get JSON for programmatic use
```bash
cargo run --bin cardex -- search "cPointElm" --index .cardex --json
```

## 🛠️ Development Commands

```bash
# Build
cargo build --workspace
cargo build --release --workspace

# Test
cargo test --workspace

# Quality checks
cargo fmt --all
cargo clippy --all

# Run CLI
cargo run --bin cardex -- --help
cargo run --bin cardex -- build --help
```

## 📚 Implementation Checklist

- [x] Task 1: Workspace Skeleton
- [x] Task 2: HHC TOC Parsing
- [x] Task 3: Page Extraction & API Cards
- [x] Task 4: Build Artifacts & DocGraph
- [x] Task 5: CLI Surface
- [x] Task 6: Related Docs CLI
- [x] Task 7: Final Verification
- [x] Tested with ETABS API data

See [MVP Plan](superpowers/plans/2026-06-02-cardex-mvp-cli.md) for details.

## 🎨 Key Technologies

- **Rust** - 2024 Edition
- **Tantivy** - Full-text search engine (BM25)
- **Scraper** - HTML parsing with CSS selectors
- **Serde/JSON** - Serialization
- **Clap** - CLI argument parsing

## 📦 Generated Artifacts

When you build an index, Cardex creates:

```
.cardex/
├── pages.jsonl          # Line-delimited API cards
├── tantivy/             # Full-text search index
├── docgraph.json        # API relationships
└── manifest.json        # Metadata
```

## 🔗 Related Resources

- [ETABS Documentation](C:\Work\Code\etabApi\CSI_API_ETABS_v1_html)
- [Cardex Source Code](../crates/)
- [Tantivy Search](https://tantivy-search.github.io/)

## 📞 Next Steps

1. Review the [Project Overview](superpowers/README.md)
2. Check the [Build Report](superpowers/build-report-etabs-api.md) for API inventory
3. Try the [Quick Start](#-getting-started) commands
4. Explore specific interfaces with the `members` command

---

**Status**: MVP Complete ✅  
**Build Date**: 2026-06-02  
**Pages Indexed**: 1,776  
**Search Ready**: Yes
