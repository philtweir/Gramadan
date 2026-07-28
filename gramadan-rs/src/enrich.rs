//! Grammar class enrichment for terminology pipelines.
//!
//! Provides a single entry point [`enrich_grammar_class`] that assigns a
//! declension (grammar class) to Irish nouns and a conjugation class to verbs,
//! using a cascade of strategies from authoritative records down to
//! morphological heuristics.
//!
//! # Strategy cascade (nouns)
//!
//! 1. **Already stated** — the record already has a grammar_class → keep it
//! 2. **LemmaDb lookup** — BuNaMo/Kaikki exact match → use its declension
//! 3. **Heuristics**:
//!    - Vowel-ending → 4th (99%+ accurate on Téarma)
//!    - Verbal noun suffix `-adh`/`-eadh` → 1st
//!    - Verbal noun suffix `-áil`/`-ail` → 2nd
//!    - Uppercase initial (proper noun) → 4th
//! 4. **Compound guesser** — decompose via lenition/hyphen/suffix → inherit head's declension
//! 5. **Morphological guesser** — rules based on gender + ending pattern
//!
//! # Strategy cascade (verbs)
//!
//! 1. Already stated → keep
//! 2. Lemma-only heuristic (94.6% accurate)

use crate::features::Gender;
use crate::noun::{self, Declension, LemmaDb};
use crate::verb;

/// A noun/verb record to be enriched. Mirrors the fields available in
/// both the Rust TBX parser and the Python pipeline.
#[derive(Debug, Clone)]
pub struct Record {
    /// The headword (lemma).
    pub word: String,
    /// Part of speech: "noun", "verb", "adjective", etc.
    pub pos: String,
    /// Gender if known: "masculine" or "feminine" (or empty).
    pub gender: String,
    /// Grammar class if already stated: "1".."5" for nouns, "1"/"2"/"irr" for verbs.
    /// Empty string means unknown.
    pub grammar_class: String,
}

/// Result of enrichment for a single record.
#[derive(Debug, Clone)]
pub struct EnrichResult {
    /// The assigned grammar class ("1".."5", "irr", or "" if unresolvable).
    pub grammar_class: String,
    /// Which strategy resolved it.
    pub method: Method,
}

/// Which strategy resolved the grammar class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Already had a stated class — kept as-is.
    AlreadyStated,
    /// Exact match in LemmaDb (BuNaMo/Kaikki).
    DbLookup,
    /// Vowel-ending heuristic → 4th.
    HeuristicVowel4th,
    /// Verbal noun `-adh`/`-eadh` → 1st.
    HeuristicVnAdh,
    /// Verbal noun `-áil`/`-ail` → 2nd.
    HeuristicVnAil,
    /// Proper noun (uppercase) → 4th.
    HeuristicProper4th,
    /// Compound decomposition (lenition/hyphen/suffix).
    CompoundDecomposition,
    /// Morphological rules (gender + ending pattern).
    MorphologicalGuesser,
    /// Verb conjugation heuristic.
    VerbHeuristic,
    /// Could not determine (no gender, unsupported POS, etc.).
    Unresolved,
}

const VOWELS: &str = "aeiouáéíóú";

fn parse_gender(s: &str) -> Option<Gender> {
    match s {
        "masculine" | "masc" => Some(Gender::Masc),
        "feminine" | "fem" => Some(Gender::Fem),
        _ => None,
    }
}

/// Enrich grammar class for a single record.
///
/// `db` may be empty if no BuNaMo/Kaikki data is available — the function
/// will still apply heuristics and the morphological guesser.
pub fn enrich_grammar_class(record: &Record, db: &LemmaDb) -> EnrichResult {
    // Already stated
    if !record.grammar_class.is_empty() {
        return EnrichResult {
            grammar_class: record.grammar_class.clone(),
            method: Method::AlreadyStated,
        };
    }

    match record.pos.as_str() {
        "noun" => enrich_noun(record, db),
        "verb" => enrich_verb(record),
        _ => EnrichResult {
            grammar_class: String::new(),
            method: Method::Unresolved,
        },
    }
}

fn enrich_noun(record: &Record, db: &LemmaDb) -> EnrichResult {
    let lemma = record.word.trim();
    if lemma.is_empty() {
        return EnrichResult {
            grammar_class: String::new(),
            method: Method::Unresolved,
        };
    }

    let gender = parse_gender(&record.gender);

    // 1. LemmaDb lookup (BuNaMo/Kaikki exact match)
    if let Some(dec) = db.lookup(lemma) {
        return EnrichResult {
            grammar_class: dec.to_string(),
            method: Method::DbLookup,
        };
    }

    // Need gender for everything below
    let gender = match gender {
        Some(g) => g,
        None => {
            return EnrichResult {
                grammar_class: String::new(),
                method: Method::Unresolved,
            }
        }
    };

    let last_char = match lemma.chars().last() {
        Some(c) => c,
        None => {
            return EnrichResult {
                grammar_class: String::new(),
                method: Method::Unresolved,
            }
        }
    };

    // 2. Heuristics (high confidence, order matters)

    // Vowel-ending → 4th (99%+ on Téarma, 100% on BuNaMo classless subset)
    if VOWELS.contains(last_char) {
        return EnrichResult {
            grammar_class: "4".to_string(),
            method: Method::HeuristicVowel4th,
        };
    }

    // Verbal noun -adh/-eadh → 1st
    if lemma.ends_with("adh") || lemma.ends_with("eadh") {
        return EnrichResult {
            grammar_class: "1".to_string(),
            method: Method::HeuristicVnAdh,
        };
    }

    // Verbal noun -áil/-ail → 2nd (mostly; some are 1st, but 2nd is safer)
    if lemma.ends_with("áil") || lemma.ends_with("ail") {
        return EnrichResult {
            grammar_class: "2".to_string(),
            method: Method::HeuristicVnAil,
        };
    }

    // Proper noun → 4th
    if lemma.starts_with(|c: char| c.is_uppercase()) {
        return EnrichResult {
            grammar_class: "4".to_string(),
            method: Method::HeuristicProper4th,
        };
    }

    // 3. Compound decomposition
    if let Some(head_dec) = db.find_compound_head(lemma, gender) {
        if let Some(dec) = Declension::from_i8(head_dec) {
            return EnrichResult {
                grammar_class: dec.as_i8().to_string(),
                method: Method::CompoundDecomposition,
            };
        }
    }

    // 4. Morphological guesser (last resort)
    let dec = noun::guess_declension(lemma, gender);
    EnrichResult {
        grammar_class: dec.as_i8().to_string(),
        method: Method::MorphologicalGuesser,
    }
}

fn enrich_verb(record: &Record) -> EnrichResult {
    let lemma = record.word.trim();
    if lemma.is_empty() {
        return EnrichResult {
            grammar_class: String::new(),
            method: Method::Unresolved,
        };
    }

    let conj = verb::guess_conjugation(lemma);
    let class_str = match conj {
        verb::ConjugationClass::First => "1",
        verb::ConjugationClass::Second => "2",
        verb::ConjugationClass::Irregular => "irr",
    };

    EnrichResult {
        grammar_class: class_str.to_string(),
        method: Method::VerbHeuristic,
    }
}

/// Enrich a batch of records. Returns results in the same order.
pub fn enrich_batch(records: &[Record], db: &LemmaDb) -> Vec<EnrichResult> {
    records.iter().map(|r| enrich_grammar_class(r, db)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> LemmaDb {
        LemmaDb::from_iter(vec![
            ("bád".to_string(), 1, Gender::Masc),
            ("bróg".to_string(), 2, Gender::Fem),
            ("cloch".to_string(), 2, Gender::Fem),
            ("tarraingt".to_string(), 2, Gender::Fem),
        ])
    }

    #[test]
    fn test_already_stated() {
        let db = make_db();
        let r = Record {
            word: "cat".into(),
            pos: "noun".into(),
            gender: "masculine".into(),
            grammar_class: "1".into(),
        };
        let result = enrich_grammar_class(&r, &db);
        assert_eq!(result.grammar_class, "1");
        assert_eq!(result.method, Method::AlreadyStated);
    }

    #[test]
    fn test_db_lookup() {
        let db = make_db();
        let r = Record {
            word: "bád".into(),
            pos: "noun".into(),
            gender: "masculine".into(),
            grammar_class: String::new(),
        };
        let result = enrich_grammar_class(&r, &db);
        assert_eq!(result.grammar_class, "1");
        assert_eq!(result.method, Method::DbLookup);
    }

    #[test]
    fn test_vowel_4th() {
        let db = LemmaDb::new();
        let r = Record {
            word: "réalta".into(),
            pos: "noun".into(),
            gender: "feminine".into(),
            grammar_class: String::new(),
        };
        let result = enrich_grammar_class(&r, &db);
        assert_eq!(result.grammar_class, "4");
        assert_eq!(result.method, Method::HeuristicVowel4th);
    }

    #[test]
    fn test_verbal_noun() {
        let db = LemmaDb::new();
        let r = Record {
            word: "briseadh".into(),
            pos: "noun".into(),
            gender: "masculine".into(),
            grammar_class: String::new(),
        };
        let result = enrich_grammar_class(&r, &db);
        assert_eq!(result.grammar_class, "1");
        assert_eq!(result.method, Method::HeuristicVnAdh);
    }

    #[test]
    fn test_compound_lenition() {
        let db = make_db();
        let r = Record {
            word: "aolchloch".into(),
            pos: "noun".into(),
            gender: "feminine".into(),
            grammar_class: String::new(),
        };
        let result = enrich_grammar_class(&r, &db);
        assert_eq!(result.grammar_class, "2");
        assert_eq!(result.method, Method::CompoundDecomposition);
    }

    #[test]
    fn test_morphological_fallback() {
        let db = LemmaDb::new();
        let r = Record {
            word: "cat".into(),
            pos: "noun".into(),
            gender: "masculine".into(),
            grammar_class: String::new(),
        };
        let result = enrich_grammar_class(&r, &db);
        assert_eq!(result.grammar_class, "1");
        assert_eq!(result.method, Method::MorphologicalGuesser);
    }

    #[test]
    fn test_verb() {
        let db = LemmaDb::new();
        let r = Record {
            word: "ceannaigh".into(),
            pos: "verb".into(),
            gender: String::new(),
            grammar_class: String::new(),
        };
        let result = enrich_grammar_class(&r, &db);
        assert_eq!(result.grammar_class, "2");
        assert_eq!(result.method, Method::VerbHeuristic);
    }
}
