//! PyO3 extension module `gramadan_rs` — a thin binding over `enrich` so the
//! Gréasán Python pipeline (`tbx.py`) can fill in a `grammar_class` for classless
//! Téarma nouns/verbs at build time. Built only under `--features python`
//! (maturin drives that via pyproject); Rust-lib consumers never compile this.

use pyo3::prelude::*;

use crate::enrich::{enrich_grammar_class, Record};
use crate::features::Gender;
use crate::noun::LemmaDb;

/// A lemma → (declension, gender) lookup table, populated from BuNaMo (or any
/// source) on the Python side and passed to `enrich` for exact lookups and
/// gender-aware compound decomposition. Without it the guesser still runs on
/// heuristics + morphology alone.
#[pyclass(name = "LemmaDb")]
struct PyLemmaDb {
    inner: LemmaDb,
}

#[pymethods]
impl PyLemmaDb {
    #[new]
    fn new() -> Self {
        PyLemmaDb { inner: LemmaDb::new() }
    }

    /// Insert one lemma. `declension`: `1..5`, or `-1`/`0` for irregular. `gender`:
    /// "masculine"/"masc" or "feminine"/"fem" (anything else → masculine).
    fn insert(&mut self, lemma: String, declension: i8, gender: &str) {
        let g = match gender.to_ascii_lowercase().as_str() {
            "feminine" | "fem" | "f" => Gender::Fem,
            _ => Gender::Masc,
        };
        // BuNaMo class 0 means "outside the standard declensions" (irregular);
        // store it as the engine's -1 so a hit stringifies to the irregular marker.
        let d = if declension == 0 { -1 } else { declension };
        self.inner.insert(lemma, d, g);
    }
}

/// Enrich one word's grammar class.
///
/// Returns `(grammar_class, method)`: `grammar_class` is `"1".."5"` for nouns,
/// `"1"/"2"/"irr"` for verbs, or `""` if unresolved; `method` is the resolving
/// strategy's name for the caller to map to a confidence level. Pass a populated
/// `db` for BuNaMo-backed exact lookups; omit it for heuristics/guesser only.
#[pyfunction]
#[pyo3(signature = (word, pos, gender, grammar_class, db=None))]
fn enrich(
    word: String,
    pos: String,
    gender: String,
    grammar_class: String,
    db: Option<&PyLemmaDb>,
) -> (String, String) {
    let empty;
    let db_ref = match db {
        Some(d) => &d.inner,
        None => {
            empty = LemmaDb::new();
            &empty
        }
    };
    let rec = Record { word, pos, gender, grammar_class };
    let res = enrich_grammar_class(&rec, db_ref);
    (res.grammar_class, format!("{:?}", res.method))
}

#[pymodule]
fn gramadan_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLemmaDb>()?;
    m.add_function(wrap_pyfunction!(enrich, m)?)?;
    Ok(())
}
