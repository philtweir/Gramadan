//! Compile-time patches for irregular-verb XML in the BuNaMo `data/` submodule.
//!
//! Each `patches/<verb>_verb.xml` is an XML fragment of extra `<tenseForm>` /
//! `<moodForm>` lines to add to the matching `data/verb/<same-name>` - upstream
//! BuNaMo omits some forms gramadan needs (e.g. bí's RelIndep relative forms
//! `a bhíos / a bheas`). Rather than fork BuNaMo or edit `src/`, we inject each
//! fragment here, just before the target's closing `</verb>` (the parser collects
//! form children by attributes, so order is irrelevant). Idempotent: the first
//! non-empty line of a fragment is the "already applied" marker. Add a file to
//! `patches/` to patch another verb; see patches/README.md.
use std::{fs, path::Path};

const CLOSE: &str = "</verb>";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let patches = Path::new("patches"); // cwd = crate root (gramadan-rs/)
    if !patches.is_dir() {
        return;
    }

    for entry in fs::read_dir(patches).expect("read patches/").flatten() {
        let snippet_path = entry.path();
        if snippet_path.extension().and_then(|e| e.to_str()) != Some("xml") {
            continue; // skip README.md etc.
        }
        println!("cargo:rerun-if-changed={}", snippet_path.display());

        let name = snippet_path.file_name().unwrap().to_string_lossy().into_owned();
        let target = Path::new("../data/verb").join(&name);
        let src = fs::read_to_string(&target).unwrap_or_else(|e| {
            panic!(
                "cannot read {}: {e} — is the BuNaMo `data/` submodule checked out? (git submodule update --init --recursive)",
                target.display()
            )
        });

        let snippet = fs::read_to_string(&snippet_path).unwrap();
        let marker = snippet.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
        if !marker.is_empty() && src.contains(marker) {
            continue; // already applied (patched data/, or a re-run)
        }

        let at = src.rfind(CLOSE).unwrap_or_else(|| {
            panic!("{}: no {CLOSE} close tag — upstream BuNaMo layout changed", target.display())
        });
        let mut out = String::with_capacity(src.len() + snippet.len() + 1);
        out.push_str(&src[..at]);
        out.push_str(snippet.trim_end());
        out.push('\n');
        out.push_str(&src[at..]);
        fs::write(&target, out).expect("write patched verb xml");
    }
}
