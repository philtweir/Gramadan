// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gréasán morphology binding. `paradigm_forms(lemma, pos, gender, class)` returns
//! a JSON `{ declension, forms }` where `forms` is a flat `FormItem[]`
//! (`{writtenRep, tags}`) using BuNaMo's tag vocabulary - the exact shape
//! `app/src/lib/paradigm.ts::buildParadigm` pivots into a table.
//!
//! NOUN emits the full grid from gramadan-rs's `singular_paradigm` (nom/gen/voc/
//! dat) and `plural_paradigm` (nom/gen/voc), tagged singular/plural + case so
//! paradigm.ts lays it out as a number x case table. Unknown/irregular declensions
//! (paradigm returns None) fall back to the citation pair (lemma + generate_genitive)
//! so the grid is never empty. VERB and ADJECTIVE paradigms are still stubbed and
//! land when gramadan-rs gains them (see
//! gramadan-rs/HANDOFF-verb-adjective-paradigms.md). The tag mapping below is the
//! single place struct-slots become tags - extend it, and Gréasán renders the new
//! forms with no frontend change.

use gramadan::adjective;
use gramadan::features::{Form, Gender};
use gramadan::noun::{
    generate_genitive, guess_declension, plural_paradigm, singular_paradigm, Declension, Noun,
};
use gramadan::np::NounPhrase;
use gramadan::verb::{guess_conjugation, PersonForms, TenseForms, Verb, VerbConjugationClass};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct FormItem {
    #[serde(rename = "writtenRep")]
    written_rep: String,
    tags: Vec<&'static str>,
}

#[derive(Serialize)]
struct Paradigm {
    /// 1..5, or 0 when unknown/irregular. Informational for the UI.
    declension: i8,
    /// Whether this POS is generated yet (false ⇒ verb/adjective placeholder).
    supported: bool,
    forms: Vec<FormItem>,
}

fn gender_of(s: &str) -> Gender {
    let s = s.to_ascii_lowercase();
    if s.starts_with('f') || s.contains("bain") {
        Gender::Fem
    } else {
        Gender::Masc
    }
}

fn decl_i8(d: Declension) -> i8 {
    match d {
        Declension::First => 1,
        Declension::Second => 2,
        Declension::Third => 3,
        Declension::Fourth => 4,
        Declension::Fifth => 5,
        Declension::Irregular => 0,
    }
}

/// Push one FormItem per surface form, skipping blanks. `number`/`case` are the
/// BuNaMo axis tags paradigm.ts pivots on. Multiple forms in a slot (variants)
/// become separate cells; paradigm.ts dedups by writtenRep within a cell.
fn push_forms(out: &mut Vec<FormItem>, forms: &[Form], number: &'static str, case: &'static str) {
    for f in forms {
        let v = f.value.trim();
        if v.is_empty() {
            continue;
        }
        out.push(FormItem {
            written_rep: v.to_string(),
            tags: vec![number, case],
        });
    }
}

fn noun_paradigm(lemma: &str, gender: &str, class: &str) -> Paradigm {
    let g = gender_of(gender);
    // Honour the entry's stated class when given (1..5); otherwise let gramadan guess.
    let decl: i8 = class
        .trim()
        .parse::<i8>()
        .ok()
        .filter(|d| (1..=5).contains(d))
        .unwrap_or_else(|| guess_declension(lemma, g).map(decl_i8).unwrap_or(0));

    let mut forms: Vec<FormItem> = Vec::new();

    match singular_paradigm(lemma, g, decl) {
        Some(si) => {
            push_forms(&mut forms, &si.nominative, "singular", "nominative");
            push_forms(&mut forms, &si.genitive, "singular", "genitive");
            push_forms(&mut forms, &si.vocative, "singular", "vocative");
            push_forms(&mut forms, &si.dative, "singular", "dative");
        }
        None => {
            // Irregular / unguessable declension: fall back to the citation pair so
            // the grid is never empty.
            forms.push(FormItem {
                written_rep: lemma.to_string(),
                tags: vec!["singular", "nominative"],
            });
            if let Some(gen) = generate_genitive(lemma, g, decl) {
                forms.push(FormItem {
                    written_rep: gen,
                    tags: vec!["singular", "genitive"],
                });
            }
        }
    }

    // The nominative singular must always carry the headword, even if the paradigm
    // strategy left it empty - otherwise the table loses its anchor row.
    if !forms
        .iter()
        .any(|f| f.tags == ["singular", "nominative"])
    {
        forms.insert(
            0,
            FormItem {
                written_rep: lemma.to_string(),
                tags: vec!["singular", "nominative"],
            },
        );
    }

    if let Some(pi) = plural_paradigm(lemma, g, decl) {
        push_forms(&mut forms, &pi.nominative, "plural", "nominative");
        push_forms(&mut forms, &pi.genitive, "plural", "genitive");
        push_forms(&mut forms, &pi.vocative, "plural", "vocative");
    }

    // Definite (articled) forms - "an bhróg" / "na mbróg" - tagged `definite` so the
    // frontend article toggle can swap them in. Nom/gen sg+pl only (NounPhrase does
    // not yet cover voc/dat). The Noun is rebuilt from the same lemma+decl.
    let np = NounPhrase::from_noun(&Noun::from_lemma_with_declension(lemma, g, decl));
    let def = |out: &mut Vec<FormItem>, forms: &[Form], number: &'static str, case: &'static str| {
        for f in forms {
            let v = f.value.trim();
            if !v.is_empty() {
                out.push(FormItem { written_rep: v.to_string(), tags: vec![number, case, "definite"] });
            }
        }
    };
    // sg_*_art are FormSg (carry gender); map to bare Form for the shared helper.
    let sg_nom_art: Vec<Form> = np.sg_nom_art.iter().map(|f| Form::new(&f.value)).collect();
    let sg_gen_art: Vec<Form> = np.sg_gen_art.iter().map(|f| Form::new(&f.value)).collect();
    def(&mut forms, &sg_nom_art, "singular", "nominative");
    def(&mut forms, &sg_gen_art, "singular", "genitive");
    def(&mut forms, &np.pl_nom_art, "plural", "nominative");
    def(&mut forms, &np.pl_gen_art, "plural", "genitive");

    Paradigm {
        declension: decl,
        supported: true,
        forms,
    }
}

/// Map the entry's stated verb class ("1"/"2"/"irr") to gramadan's conjugation
/// class; guess from the lemma when unstated.
fn verb_class(class: &str, lemma: &str) -> VerbConjugationClass {
    match class.trim() {
        "1" => VerbConjugationClass::First,
        "2" => VerbConjugationClass::Second,
        "irr" | "0" => VerbConjugationClass::Irregular,
        _ => guess_conjugation(lemma),
    }
}

/// BuNaMo-style person tags for a PersonForms slot. Base (analytic) carries no
/// person tag; Auto is the autonomous form. Mirrors build-bunamo-data.py's
/// PERSON_MAP so both grammar tabs pivot identically in paradigm.ts.
fn person_tags(person: &str) -> Vec<&'static str> {
    match person {
        "sg1" => vec!["singular", "first-person"],
        "sg2" => vec!["singular", "second-person"],
        "sg3" => vec!["singular", "third-person"],
        "pl1" => vec!["plural", "first-person"],
        "pl2" => vec!["plural", "second-person"],
        "pl3" => vec!["plural", "third-person"],
        "auto" => vec!["autonomous"],
        _ => vec![],
    }
}

/// Emit one FormItem per person slot of a PersonForms, tagged with `base_tags`
/// (tense/mood + "indicative") plus the person tags, and "dependent" for the
/// dependent set (the frontend independent/dependent toggle filters on it).
fn push_person(out: &mut Vec<FormItem>, pf: &PersonForms, base_tags: &[&'static str], dep: bool) {
    let slots: [(&str, &Vec<Form>); 8] = [
        ("base", &pf.base),
        ("sg1", &pf.sg1),
        ("sg2", &pf.sg2),
        ("sg3", &pf.sg3),
        ("pl1", &pf.pl1),
        ("pl2", &pf.pl2),
        ("pl3", &pf.pl3),
        ("auto", &pf.auto),
    ];
    for (key, forms) in slots {
        for f in forms {
            let v = f.value.trim();
            if v.is_empty() {
                continue;
            }
            let mut tags: Vec<&'static str> = base_tags.to_vec();
            tags.extend(person_tags(key));
            if dep {
                tags.push("dependent");
            }
            out.push(FormItem { written_rep: v.to_string(), tags });
        }
    }
}

/// Learner-core verb conjugation: verbal noun/adjective, past/present/future/
/// conditional (each with independent + dependent sets) and the imperative. The
/// habitual and subjunctive tenses are omitted for now. Dependent forms carry a
/// "dependent" tag so the frontend can toggle independent vs dependent.
fn verb_paradigm(lemma: &str, class: &str) -> Paradigm {
    let vc = verb_class(class, lemma);
    let v = Verb::from_lemma(lemma, vc);
    let mut forms: Vec<FormItem> = Vec::new();

    for f in &v.verbal_noun {
        let val = f.value.trim();
        if !val.is_empty() {
            forms.push(FormItem { written_rep: val.to_string(), tags: vec!["verbal-noun"] });
        }
    }
    for f in &v.verbal_adjective {
        let val = f.value.trim();
        if !val.is_empty() {
            forms.push(FormItem { written_rep: val.to_string(), tags: vec!["verbal-adjective"] });
        }
    }

    // "present" maps to gramadan's present-habitual (pres_cont), the everyday
    // present for all verbs bar the copula/ta.
    let tenses: [(&TenseForms, &'static str); 4] = [
        (&v.past, "past"),
        (&v.pres_cont, "present"),
        (&v.fut, "future"),
        (&v.cond, "conditional"),
    ];
    for (tf, tense) in tenses {
        push_person(&mut forms, &tf.indep, &[tense, "indicative"], false);
        push_person(&mut forms, &tf.dep, &[tense, "indicative"], true);
    }
    // Imperative has no independent/dependent split.
    push_person(&mut forms, &v.imper, &["imperative"], false);

    let conj = match vc {
        VerbConjugationClass::First => 1,
        VerbConjugationClass::Second => 2,
        VerbConjugationClass::Irregular => 0,
    };
    Paradigm { declension: conj, supported: true, forms }
}

/// Map the entry's stated adjective class ("1".."3") to a declension; guess otherwise.
fn adj_class(class: &str, lemma: &str) -> Declension {
    match class.trim() {
        "1" => Declension::First,
        "2" => Declension::Second,
        "3" => Declension::Third,
        _ => adjective::guess_declension(lemma),
    }
}

fn push_adj(out: &mut Vec<FormItem>, v: &str, tags: Vec<&'static str>) {
    let v = v.trim();
    if !v.is_empty() {
        out.push(FormItem { written_rep: v.to_string(), tags });
    }
}

/// Learner-core adjective declension: nominative singular, genitive singular
/// (masculine + feminine - the gender split), nominative plural, and the graded
/// (comparative/superlative) form. Vocative is skipped (not attested in BuNaMo,
/// so both tabs stay consistent). Tagged in BuNaMo's vocabulary; gender rides as a
/// cell qualifier so paradigm.ts shows masc/fem genitive side by side.
fn adjective_paradigm(lemma: &str, class: &str) -> Paradigm {
    let decl = adj_class(class, lemma);
    let mut forms: Vec<FormItem> = Vec::new();
    if let Some(a) = adjective::generate_forms(lemma, decl) {
        push_adj(&mut forms, &a.sg_nom, vec!["singular", "nominative"]);
        push_adj(&mut forms, &a.sg_gen_masc, vec!["singular", "genitive", "masculine"]);
        push_adj(&mut forms, &a.sg_gen_fem, vec!["singular", "genitive", "feminine"]);
        push_adj(&mut forms, &a.pl_nom, vec!["plural", "nominative"]);
        push_adj(&mut forms, &a.graded, vec!["comparative", "superlative"]);
    }
    // Anchor row: the nominative singular always carries the headword.
    if !forms.iter().any(|f| f.tags == ["singular", "nominative"]) {
        forms.insert(0, FormItem { written_rep: lemma.to_string(), tags: vec!["singular", "nominative"] });
    }
    let d = match decl {
        Declension::First => 1,
        Declension::Second => 2,
        Declension::Third => 3,
        _ => 0,
    };
    Paradigm { declension: d, supported: true, forms }
}

/// Generate a paradigm for one headword. `pos` ∈ {"noun","verb","adjective"};
/// `gender` is the gender label ("masculine"/"feminine"/…, empty ok); `class` is
/// the stated grammatical class ("1".."5", or empty to guess). Returns JSON.
#[wasm_bindgen]
pub fn paradigm_forms(lemma: &str, pos: &str, gender: &str, class: &str) -> String {
    let p = match pos.to_ascii_lowercase().as_str() {
        "noun" => noun_paradigm(lemma, gender, class),
        "verb" => verb_paradigm(lemma, class),
        "adjective" | "adj" => adjective_paradigm(lemma, class),
        _ => Paradigm {
            declension: 0,
            supported: false,
            forms: vec![],
        },
    };
    serde_json::to_string(&p).unwrap_or_else(|_| "{\"supported\":false,\"forms\":[]}".to_string())
}
