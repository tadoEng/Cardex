#![forbid(unsafe_code)]

mod build;
mod cards;
mod hhc;
mod model;
mod search;
mod store;

pub use build::build_corpus;
pub use cards::build_card_from_html;
pub use hhc::parse_hhc;
pub use model::{
    ApiCard, BuildOptions, BuildReport, CardexError, DocGraph, Manifest, PageKind, Parameter,
    Result, SearchHit, Toc, TocEntry,
};
pub use store::CardStore;
