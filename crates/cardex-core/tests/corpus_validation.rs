use std::path::PathBuf;

use cardex_core::{BuildOptions, CardStore, build_corpus};

fn corpus_dir() -> Option<PathBuf> {
    std::env::var_os("CARDEX_CORPUS_DIR").map(PathBuf::from)
}

#[test]
#[ignore = "requires the licensed CSI corpus via CARDEX_CORPUS_DIR"]
fn real_corpus_has_no_corrupted_or_collided_symbols() {
    let Some(src) = corpus_dir() else {
        return;
    };
    let temp = tempfile::tempdir().expect("temp dir");
    let out = temp.path().join("idx");

    build_corpus(BuildOptions {
        source_dir: src,
        out_dir: out.clone(),
        corpus: "etabs-api".into(),
    })
    .expect("real corpus builds");
    let store = CardStore::open(&out).expect("store opens");

    assert!(store.get("cFunctionRS.GetNTC2008").unwrap().is_some());
    assert!(store.get("cFunctionRS.GetNTC2018").unwrap().is_some());
    assert!(store.get("cFunctionRS.GetNTC").unwrap().is_none());

    let overloads = store
        .overloads("cAnalysisResults.AssembledJointMass")
        .expect("overloads load");
    assert!(
        overloads
            .iter()
            .any(|symbol| symbol.ends_with(".AssembledJointMass"))
    );
    assert!(
        overloads
            .iter()
            .any(|symbol| symbol.ends_with(".AssembledJointMass_1"))
    );
}
