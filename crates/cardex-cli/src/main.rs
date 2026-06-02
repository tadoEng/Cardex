#![forbid(unsafe_code)]

use std::path::PathBuf;

use anyhow::{Context, bail};
use cardex_core::{BuildOptions, CardStore, build_corpus};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "cardex")]
#[command(about = "Local/offline API-card retrieval for engineering software agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Build {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value = "etabs-api")]
        corpus: String,
        #[arg(long)]
        json: bool,
    },
    Search {
        query: String,
        #[arg(long, default_value = ".cardex/etabs-api")]
        index: PathBuf,
        #[arg(long, default_value_t = 8)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    Get {
        symbol: String,
        #[arg(long, default_value = ".cardex/etabs-api")]
        index: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Members {
        interface: String,
        #[arg(long, default_value = ".cardex/etabs-api")]
        index: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build {
            source,
            out,
            corpus,
            json,
        } => {
            let report = build_corpus(BuildOptions {
                source_dir: source,
                out_dir: out,
                corpus,
            })
            .context("failed to build Cardex corpus")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "Built {} pages for {} at {}",
                    report.pages,
                    report.corpus,
                    report.output_dir.display()
                );
            }
        }
        Command::Search {
            query,
            index,
            limit,
            json,
        } => {
            let store = CardStore::open(&index).context("failed to open Cardex index")?;
            let hits = store.search(&query, limit).context("search failed")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&hits)?);
            } else {
                for hit in hits {
                    println!(
                        "{:.3}\t{}\t{}",
                        hit.score,
                        hit.symbol.as_deref().unwrap_or(&hit.page_id),
                        hit.summary.as_deref().unwrap_or(&hit.title)
                    );
                }
            }
        }
        Command::Get {
            symbol,
            index,
            json,
        } => {
            let store = CardStore::open(&index).context("failed to open Cardex index")?;
            let Some(card) = store.get(&symbol).context("get failed")? else {
                bail!("no card found for {symbol}");
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&card)?);
            } else {
                println!("{}", card.symbol.as_deref().unwrap_or(&card.title));
                if let Some(signature) = card.signature_cs.as_deref() {
                    println!("C#: {signature}");
                }
                if let Some(returns) = card.returns.as_deref() {
                    println!("Returns: {returns}");
                }
                if let Some(remarks) = card.remarks.as_deref() {
                    println!("Remarks: {remarks}");
                }
            }
        }
        Command::Members {
            interface,
            index,
            json,
        } => {
            let store = CardStore::open(&index).context("failed to open Cardex index")?;
            let members = store.members(&interface).context("members failed")?;
            if json {
                println!("{}", serde_json::to_string_pretty(&members)?);
            } else {
                for member in members {
                    println!("{member}");
                }
            }
        }
    }

    Ok(())
}
