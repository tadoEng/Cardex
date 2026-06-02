# Cardex Documentation

## Overview

Cardex is a Rust-based tool for indexing and searching API documentation from decompiled CHM (Compiled HTML) archives. This implementation demonstrates building a searchable corpus for the ETABS structural engineering API.

## Documentation

### Plans
- [Cardex MVP CLI Implementation Plan](plans/2026-06-02-cardex-mvp-cli.md) - Original specification and task breakdown

### Implementation Reports
- [ETABS API Build Report](build-report-etabs-api.md) - Completed ETABS indexing reports and smoke-test counts

## Quick Start

### Building the Index
```bash
cargo run --bin cardex -- build \
  --source "C:\Work\Code\etabApi\CSI_API_ETABS_v1_html" \
  --out ".cardex" \
  --corpus etabs-api
```

### Searching the Index
```bash
# Search for API documentation
cargo run --bin cardex -- search "cPointElm" --index .cardex --limit 5

# List interface members
cargo run --bin cardex -- members "cPointElm" --index .cardex --json

# Follow related documentation
cargo run --bin cardex -- related "cAnalysisResults.FrameForce" --index .cardex --json
```

## Key Features

- **HHC Parser**: Parses nested table of contents from HTML Help files
- **HTML Extraction**: Extracts structured API information from documentation HTML
- **Full-Text Search**: BM25-based search using Tantivy
- **JSON API**: Machine-readable output for integration with agents/tools
- **Relationship Mapping**: Tracks API relationships and interface hierarchies
- **Related Navigation**: Follows compact See Also targets from generated DocGraph edges

## Project Structure

```
.
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── cardex-core/                    # Core library
│   │   ├── src/
│   │   │   ├── lib.rs                  # Public API
│   │   │   ├── model.rs                # Data structures
│   │   │   ├── hhc.rs                  # TOC parser
│   │   │   ├── cards.rs                # HTML extraction
│   │   │   ├── build.rs                # Index building
│   │   │   ├── store.rs                # Index loader
│   │   │   └── search.rs               # Search engine
│   │   └── tests/                      # Test suite
│   └── cardex-cli/                     # CLI tool
│       ├── src/
│       │   └── main.rs                 # CLI interface
│       └── tests/                      # CLI tests
├── docs/                               # Documentation
└── .cardex/                            # Generated index
    ├── pages.jsonl                     # Indexed pages
    ├── docgraph.json                   # API relationships
    ├── manifest.json                   # Metadata
    └── tantivy/                        # Search index
```

## Technology Stack

- **Language**: Rust 2024 Edition
- **Search**: Tantivy 0.26.1 (BM25 indexing)
- **HTML Parsing**: scraper 0.27.0
- **JSON**: serde_json 1.0.150
- **CLI**: clap 4.6.1 with derive macros
- **Error Handling**: thiserror 2.0.18

## Commands

### Build
```bash
cardex build --source <DIR> --out <DIR> --corpus <NAME>
```
Build a searchable index from CHM HTML directory.

### Search
```bash
cardex search <QUERY> --index <DIR> --limit <N> [--json]
```
Full-text search with BM25 ranking.

### Get
```bash
cardex get <SYMBOL> --index <DIR> [--json]
```
Retrieve specific API card by symbol.

### Members
```bash
cardex members <INTERFACE> --index <DIR> [--json]
```
List all members of an interface.

### Related
```bash
cardex related <SYMBOL> --index <DIR> [--json]
```
List See Also targets for a symbol or page.

## Development

### Run Tests
```bash
cargo test --workspace
```

### Format Code
```bash
cargo fmt --all
```

### Check Quality
```bash
cargo clippy --all
```

### Build Release
```bash
cargo build --release --workspace
```

## Performance

- Index build: ~17-34 seconds depending on compile cache and corpus snapshot
- Search latency: <100ms
- Index size: ~500MB
- Supported queries: Arbitrary text, wildcards via BM25

## Future Enhancements

1. **Search Features**
   - Fuzzy matching for typos
   - Faceted search by interface
   - Query expansion

2. **Export Formats**
   - Markdown generation
   - Integration with documentation systems
   - Agent-friendly APIs

3. **Multiple Corpora**
   - Support multiple API indices
   - Cross-corpus search
   - Corpus switching

4. **API Server**
   - REST API for remote querying
   - WebSocket support
   - Python/C# bindings

## References

- ETABS API: Computers & Structures, Inc.
- Tantivy: [https://tantivy-search.github.io/](https://tantivy-search.github.io/)
- Scraper: HTML parsing with CSS selectors
- Rust: Edition 2024

---

**Status**: MVP Complete ✅  
**Last Updated**: 2026-06-02  
**Maintainer**: Cardex Contributors
