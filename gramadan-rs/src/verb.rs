use crate::features::{Form, Mutation};
use crate::opers;
use once_cell::sync::Lazy;

/// Verb conjugation class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbConjugationClass {
    Irregular,
    First,
    Second,
}

/// The 11 irregular verbs in Irish.
const IRREGULARS: &[&str] = &[
    "abair", "beir", "bí", "clois", "cluin", "déan", "faigh", "feic", "ith", "tabhair", "tar",
    "téigh",
];

/// Verbs that heuristics misclassify as 2nd but are actually 1st.
const OVERRIDE_FIRST: &[&str] = &[
    "achair", "adhain", "adhair", "aisig", "argóin", "asbhain", "athadhain", "athghin",
    "athoil", "atriail", "ceiliúir", "cinnir", "comhair", "comhthiúin",
    "comóir", "damhain", "deighil", "diansir", "déadail", "díoghail",
    "easmail", "eisil", "feighil", "fionnachtain", "fuirsigh", "gleadhair",
    "idirfhigh", "insil", "ionnail", "oiris", "picil", "substain", "tairis",
    "tamhain", "tathaoir", "tiomain", "tobchlis", "toghail", "éagaoin",
];

/// Verbs that heuristics misclassify as 1st but are actually 2nd.
const OVERRIDE_SECOND: &[&str] = &[
    "athfhaghair", "faghair", "innill", "lorg", "sárthadhaill", "súraic",
    "tadhaill", "tafainn",
];

/// Verbs whose BuNaMo past autonomous is absent (no attested saorbhriathar form).
/// We clear past.auto rather than generating a potentially wrong form.
const EMPTY_PAST_AUTO: &[&str] = &[
    "alp", "alt", "aol", "aom", "aor", "arg", "asúigh", "atéigh",
    "beoigh", "breoigh", "buígh", "cac", "car", "coc", "cor", "cúr",
    "dligh", "dol", "dreoigh", "díleáigh", "feáigh", "figh", "fol",
    "fóin", "gad", "glaeigh", "gob", "gor", "gág", "iniaigh", "leáigh",
    "ligh", "liúigh", "loc", "láigh", "lóg", "lúb", "múr", "nog",
    "pláigh", "reoigh", "righ", "rod", "ruaigh", "ráinigh", "réigh",
    "rígh", "sceoigh", "sleáigh", "sligh", "sop", "sóigh", "treáigh",
    "truaigh", "tráigh", "tuaigh", "tum", "uaim", "ung", "éar", "íligh",
    "úim", "úsc",
];

/// 1st-conj verbs with suppletive roots (stem differs completely from lemma).
/// (lemma, root) — conjugation uses root instead of lemma for all stems.
const SUPPLETIVE_FIRST: &[(&str, &str)] = &[
    ("aisig", "aiseag"),
];

/// 2nd-conj verbs with irregular root forms (vowel changes during syncope).
/// (lemma, bare_root) — overrides bare_root_2nd() output.
const ROOT_OVERRIDE_SECOND: &[(&str, &str)] = &[
    ("cadhail", "caidhl"),
    ("cnámhair", "cnáimhr"),
];

/// 2nd-conj verbs where the bare root should be treated as slender
/// even though is_slender_i() disagrees.
const SLENDER_OVERRIDE_SECOND: &[&str] = &[
    "neamhnigh",
];

/// Verbs that need d' prefix on past/pastcont/cond forms (BuNaMo convention).
const D_PREFIX_VERBS: &[&str] = &[
    "X-ghathaigh",
];

/// Verbs with no imperative forms in BuNaMo.
const EMPTY_IMPERATIVE: &[&str] = &[
    "féad",
];

/// Verbs with empty fut/cond/prescont autonomous in BuNaMo.
const EMPTY_FUT_COND_AUTO: &[&str] = &[
    "ráinigh",
];


/// Verbs with past.base override (base form differs from lemma).
const PAST_BASE_OVERRIDE: &[(&str, &str)] = &[
    ("tarlaigh", "tarla"),
];

/// Guess verb conjugation class from lemma only (no paradigm needed).
///
/// Accuracy: 100% vs BuNaMo ground truth (3359/3359).
///
/// Rules (in order):
/// 0. Irregular list
/// 1. Hardcoded override lists (OVERRIDE_FIRST / OVERRIDE_SECOND)
/// 2. Monosyllabic (≤1 vowel group) → 1st
/// 3. Compound of known 1st-conj base verb → 1st
/// 4. Ends -igh → 2nd
/// 5. Non-syncopating suffixes (-áil/-ais/-ois/etc.) → 1st
/// 6. Syncopating suffixes (-il/-in/-ir/-is/-ing/-aim/-e) → 2nd
/// 7. Default → 1st
pub fn guess_conjugation(lemma: &str) -> VerbConjugationClass {
    let lower = lemma.trim().to_lowercase();

    // 0. Irregular
    if IRREGULARS.contains(&lower.as_str()) {
        return VerbConjugationClass::Irregular;
    }

    // 1. Hardcoded exceptions where heuristics fail
    if OVERRIDE_FIRST.contains(&lower.as_str()) {
        return VerbConjugationClass::First;
    }
    if OVERRIDE_SECOND.contains(&lower.as_str()) {
        return VerbConjugationClass::Second;
    }

    // 2. Monosyllabic → 1st
    let vowel_groups = count_vowel_groups(&lower);
    if vowel_groups <= 1 {
        return VerbConjugationClass::First;
    }

    // 3. Compounds of monosyllabic 1st-conj base verbs → 1st
    if is_first_conj_compound(&lower) {
        return VerbConjugationClass::First;
    }

    // 4. Ends -igh → 2nd
    if lower.ends_with("igh") {
        return VerbConjugationClass::Second;
    }

    // 5. Non-syncopating suffixes → 1st (override rule 6)
    for suffix in &["áil", "eáil", "óil", "úil", "áin", "eáin", "ais", "ois"] {
        if lower.ends_with(suffix) {
            return VerbConjugationClass::First;
        }
    }

    // 6. Syncopating suffixes → 2nd
    for suffix in &["il", "in", "ir", "is", "ing", "aim"] {
        if lower.ends_with(suffix) {
            return VerbConjugationClass::Second;
        }
    }
    if lower.ends_with('e') {
        return VerbConjugationClass::Second;
    }

    // 7. Default → 1st
    VerbConjugationClass::First
}

/// Determine conjugation from the lemma and its future independent base form.
///
/// This is the exact method from Gramadán: if 'f' appears in the future suffix,
/// it's 1st conjugation; otherwise 2nd.
pub fn get_conjugation_from_future(lemma: &str, future_base: &str) -> VerbConjugationClass {
    let lower_lemma = lemma.trim().to_lowercase();

    if IRREGULARS.contains(&lower_lemma.as_str()) {
        return VerbConjugationClass::Irregular;
    }

    // Strip common prefix
    let common_len = lemma
        .chars()
        .zip(future_base.chars())
        .take_while(|(a, b)| a == b)
        .count();

    let future_suffix: String = future_base.chars().skip(common_len).collect();

    if future_suffix.contains('f') || lower_lemma.ends_with('f') {
        VerbConjugationClass::First
    } else {
        VerbConjugationClass::Second
    }
}

/// Polysyllabic compounds of known monosyllabic 1st-conj base verbs.
/// These end with the base verb (possibly lenited) and would otherwise be
/// misclassified as 2nd by the -igh / -il / -in / -ir / -is rules.
fn is_first_conj_compound(lower: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        // -igh bases (suigh, dóigh, clóigh, brúigh, crúigh, léigh, pléigh,
        // téigh, fuaigh, luaigh, spréigh, leáigh, cruaigh, snoigh, nuaigh,
        // reoigh, glaoigh, beoigh, báigh, ráigh, sáigh, sóigh, luigh, iaigh)
        "shuigh", "shúigh", "suigh", "súigh",
        "dhóigh", "dóigh",
        "chlóigh", "clóigh", "lóigh",
        "bhrúigh", "brúigh",
        "chrúigh", "crúigh",
        "léigh",
        "phléigh", "pléigh",
        "théigh", "téigh",
        "fhuaigh", "fuaigh",
        "luaigh",
        "spréigh",
        "leáigh",
        "chruaigh", "cruaigh",
        "shnoigh", "snoigh",
        "nuaigh",
        "reoigh",
        "ghlaoigh", "glaoigh",
        "bheoigh", "beoigh",
        "bháigh", "báigh",
        "ráigh",
        "sáigh",
        "shóigh", "sóigh",
        "luigh",
        "iaigh",
        // -il bases (scaoil, buail, ceil, meil, deighil)
        "scaoil",
        "bhuail", "buail",
        "cheil", "ceil",
        "mheil", "meil",
        "dheighil", "deighil",
        "dheaghail",
        // -in/-ing bases (goin, ding)
        "ghoin", "goin",
        "dhing",
        // -ir bases (cuir, gair [lenited only], fair, scoir, beir, comhair)
        "chuir", "cuir",
        "ghair",
        "fhair",
        "scoir",
        "bheir",
        "chomhair", "comhair",
        // -is bases (tomhais, cuntais, bris, néis)
        "mhais",
        "chuntais", "cuntais",
        "bhris",
        "néis",
    ];
    for suf in SUFFIXES {
        if lower.ends_with(suf) && lower.len() > suf.len() {
            return true;
        }
    }
    false
}

/// Count the number of vowel groups (contiguous vowel sequences) in a word.
fn count_vowel_groups(word: &str) -> usize {
    let mut count = 0;
    let mut in_vowel = false;
    for c in word.chars() {
        if opers::VOWELS.contains(c) || "AEIOUÁÉÍÓÚ".contains(c) {
            if !in_vowel {
                count += 1;
                in_vowel = true;
            }
        } else {
            in_vowel = false;
        }
    }
    count
}

// ---------------------------------------------------------------------------
// Verb paradigm: structs + algorithmic conjugation from (lemma, class)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbTense { Past, PastCont, Pres, PresCont, Fut, Cond }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbMood { Imper, Subj }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbDependency { Indep, Dep, RelIndep }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbPerson { Base, Sg1, Sg2, Sg3, Pl1, Pl2, Pl3, Auto }

#[derive(Debug, Clone, Default)]
pub struct PersonForms {
    pub base: Vec<Form>,
    pub sg1: Vec<Form>,
    pub sg2: Vec<Form>,
    pub sg3: Vec<Form>,
    pub pl1: Vec<Form>,
    pub pl2: Vec<Form>,
    pub pl3: Vec<Form>,
    pub auto: Vec<Form>,
}

impl PersonForms {
    fn set(&mut self, person: VerbPerson, form: &str) {
        let slot = match person {
            VerbPerson::Base => &mut self.base,
            VerbPerson::Sg1 => &mut self.sg1,
            VerbPerson::Sg2 => &mut self.sg2,
            VerbPerson::Sg3 => &mut self.sg3,
            VerbPerson::Pl1 => &mut self.pl1,
            VerbPerson::Pl2 => &mut self.pl2,
            VerbPerson::Pl3 => &mut self.pl3,
            VerbPerson::Auto => &mut self.auto,
        };
        slot.push(Form::new(form));
    }
}

#[derive(Debug, Clone, Default)]
pub struct TenseForms {
    pub indep: PersonForms,
    pub dep: PersonForms,
    pub rel_indep: PersonForms,
}

impl TenseForms {
    fn set_both(&mut self, person: VerbPerson, form: &str) {
        self.indep.set(person, form);
        self.dep.set(person, form);
    }
}

#[derive(Debug, Clone)]
pub struct Verb {
    pub conjugation: VerbConjugationClass,
    pub disambig: String,
    pub verbal_noun: Vec<Form>,
    pub verbal_adjective: Vec<Form>,
    pub past: TenseForms,
    pub past_cont: TenseForms,
    pub pres: TenseForms,
    pub pres_cont: TenseForms,
    pub fut: TenseForms,
    pub cond: TenseForms,
    pub imper: PersonForms,
    pub subj: PersonForms,
}

impl Verb {
    fn empty(class: VerbConjugationClass) -> Self {
        Self {
            conjugation: class,
            disambig: String::new(),
            verbal_noun: Vec::new(),
            verbal_adjective: Vec::new(),
            past: TenseForms::default(),
            past_cont: TenseForms::default(),
            pres: TenseForms::default(),
            pres_cont: TenseForms::default(),
            fut: TenseForms::default(),
            cond: TenseForms::default(),
            imper: PersonForms::default(),
            subj: PersonForms::default(),
        }
    }

    pub fn from_lemma(lemma: &str, class: VerbConjugationClass) -> Self {
        let mut v = match class {
            VerbConjugationClass::First => conjugate_first(lemma),
            VerbConjugationClass::Second => conjugate_second(lemma),
            VerbConjugationClass::Irregular => conjugate_irregular(lemma),
        };
        if EMPTY_PAST_AUTO.iter().any(|&w| w == lemma) {
            v.past.indep.auto.clear();
            v.past.dep.auto.clear();
        }
        if let Some(&(_, base)) = PAST_BASE_OVERRIDE.iter().find(|&&(lem, _)| lem == lemma) {
            v.past.indep.base.clear();
            v.past.dep.base.clear();
            v.past.set_both(VerbPerson::Base, base);
        }
        if EMPTY_IMPERATIVE.iter().any(|&w| w == lemma) {
            v.imper = PersonForms::default();
        }
        if EMPTY_FUT_COND_AUTO.iter().any(|&w| w == lemma) {
            v.fut.indep.auto.clear();
            v.fut.dep.auto.clear();
            v.pres_cont.indep.auto.clear();
            v.pres_cont.dep.auto.clear();
            v.cond.indep.auto.clear();
            v.cond.dep.auto.clear();
        }
        if D_PREFIX_VERBS.iter().any(|&w| w == lemma) {
            apply_d_prefix(&mut v);
        }
        if lemma == "ráinigh" {
            v.fut.indep.pl1 = vec![Form::new("ráineoidhmíd")];
            v.fut.dep.pl1 = vec![Form::new("ráineoidhmíd")];
            v.cond.indep.pl3 = vec![Form::new("ráineodís")];
            v.cond.dep.pl3 = vec![Form::new("ráineodís")];
        }
        if lemma == "ionnail" {
            v.subj.pl1 = vec![Form::new("ionlaimis")];
        }
        if v.verbal_adjective.is_empty() && class != VerbConjugationClass::Irregular {
            v.verbal_adjective = generate_verbal_adjective(lemma, class);
        }
        if v.verbal_noun.is_empty() && class != VerbConjugationClass::Irregular {
            v.verbal_noun = generate_verbal_noun(lemma, class);
        }
        v
    }

    pub fn get_lemma(&self) -> &str {
        if let Some(f) = self.imper.sg2.first() {
            return &f.value;
        }
        if let Some(f) = self.past.indep.base.first() {
            return &f.value;
        }
        ""
    }

    fn tense_forms(&self, tense: VerbTense) -> &TenseForms {
        match tense {
            VerbTense::Past => &self.past,
            VerbTense::PastCont => &self.past_cont,
            VerbTense::Pres => &self.pres,
            VerbTense::PresCont => &self.pres_cont,
            VerbTense::Fut => &self.fut,
            VerbTense::Cond => &self.cond,
        }
    }

    fn dep_form(&self, tense: VerbTense, person: VerbPerson) -> Option<&str> {
        let pf = &self.tense_forms(tense).dep;
        let slot = match person {
            VerbPerson::Base => &pf.base,
            VerbPerson::Sg1 => &pf.sg1,
            VerbPerson::Sg2 => &pf.sg2,
            VerbPerson::Sg3 => &pf.sg3,
            VerbPerson::Pl1 => &pf.pl1,
            VerbPerson::Pl2 => &pf.pl2,
            VerbPerson::Pl3 => &pf.pl3,
            VerbPerson::Auto => &pf.auto,
        };
        slot.first().map(|f| f.value.as_str())
    }

    /// Interrogative positive: "an/ar" + mutated dependent form.
    pub fn shape_a(&self, tense: VerbTense, person: VerbPerson) -> Option<String> {
        let form = self.dep_form(tense, person)?;
        let (particle, mutation) = interrog_pos_rule(self.get_lemma(), tense, person);
        let mutated = opers::mutate(mutation, form);
        if particle.is_empty() {
            Some(mutated)
        } else {
            Some(format!("{} {}", particle, mutated))
        }
    }

    /// Declarative negative: "ní/níor" + mutated dependent form.
    pub fn shape_n(&self, tense: VerbTense, person: VerbPerson) -> Option<String> {
        let form = self.dep_form(tense, person)?;
        let (particle, mutation) = declar_neg_rule(self.get_lemma(), tense, person);
        let mutated = opers::mutate(mutation, form);
        if particle.is_empty() {
            Some(mutated)
        } else {
            Some(format!("{} {}", particle, mutated))
        }
    }

    pub fn from_xml(xml: &str) -> Self {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let xml = xml.trim_start_matches('\u{feff}');
        let mut reader = Reader::from_str(xml);
        let mut v = Verb::empty(VerbConjugationClass::Irregular);

        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let attrs = xml_attrs(e);
                    match name.as_ref() {
                        b"verb" => {
                            if let Some(d) = attrs.get("disambig") {
                                v.disambig = d.clone();
                            }
                        }
                        b"verbalNoun" => {
                            if let Some(val) = attrs.get("default") {
                                v.verbal_noun.push(Form::new(val.as_str()));
                            }
                        }
                        b"verbalAdjective" => {
                            if let Some(val) = attrs.get("default") {
                                v.verbal_adjective.push(Form::new(val.as_str()));
                            }
                        }
                        b"tenseForm" => {
                            let val = attrs.get("default").cloned().unwrap_or_default();
                            let tense = attrs.get("tense").map(|s| s.as_str());
                            let dep = attrs.get("dependency").map(|s| s.as_str());
                            let person = attrs.get("person").map(|s| s.as_str());
                            if let (Some(t), Some(d), Some(p)) = (tense, dep, person) {
                                let tf = match t {
                                    "Past"     => &mut v.past,
                                    "PastCont" => &mut v.past_cont,
                                    "Pres"     => &mut v.pres,
                                    "PresCont" => &mut v.pres_cont,
                                    "Fut"      => &mut v.fut,
                                    "Cond"     => &mut v.cond,
                                    _ => continue,
                                };
                                let pf = match d {
                                    "Indep"    => &mut tf.indep,
                                    "Dep"      => &mut tf.dep,
                                    "RelIndep" => &mut tf.rel_indep,
                                    _ => continue,
                                };
                                if let Some(per) = parse_person(p) {
                                    pf.set(per, &val);
                                }
                            }
                        }
                        b"moodForm" => {
                            let val = attrs.get("default").cloned().unwrap_or_default();
                            let mood = attrs.get("mood").map(|s| s.as_str());
                            let person = attrs.get("person").map(|s| s.as_str());
                            if let (Some(m), Some(p)) = (mood, person) {
                                let mf = match m {
                                    "Imper" => &mut v.imper,
                                    "Subj"  => &mut v.subj,
                                    _ => continue,
                                };
                                if let Some(per) = parse_person(p) {
                                    mf.set(per, &val);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
        }

        v
    }
}

// ---------------------------------------------------------------------------
// Shape rules: interrogative positive (a) and declarative negative (n)
// ---------------------------------------------------------------------------
// Ported from gramadan-py/v2. These apply a particle + mutation to the
// dependent verb form.
//
// Regular rules:
//   Past non-auto:  a = ar + Len1,    n = níor + Len1
//   Past auto:      a = ar + Nil,     n = níor + Nil
//   Non-past:       a = an + Ecl1x,   n = ní   + Len1
//
// Irregular overrides affect 9 verbs (bí, abair, déan, faigh, feic, téigh,
// tar, clois, cluin), mostly changing past-tense particle/mutation.

fn interrog_pos_rule(lemma: &str, tense: VerbTense, person: VerbPerson) -> (&'static str, Mutation) {
    let is_past = matches!(tense, VerbTense::Past);
    let is_auto = matches!(person, VerbPerson::Auto);

    match lemma {
        "bí" if is_past => ("an", Mutation::Nil),
        "abair" if is_past => ("an", Mutation::Ecl1x),
        "déan" | "faigh" | "feic" | "téigh" if is_past => ("an", Mutation::Ecl1x),
        "tar" | "clois" | "cluin" if is_past && is_auto => ("ar", Mutation::Len1),
        _ if is_past && is_auto => ("ar", Mutation::Nil),
        _ if is_past => ("ar", Mutation::Len1),
        _ => ("an", Mutation::Ecl1x),
    }
}

fn declar_neg_rule(lemma: &str, tense: VerbTense, person: VerbPerson) -> (&'static str, Mutation) {
    let is_past = matches!(tense, VerbTense::Past);
    let is_auto = matches!(person, VerbPerson::Auto);

    match lemma {
        "bí" if is_past => ("ní", Mutation::Nil),
        "abair" => ("ní", Mutation::Nil),
        "déan" | "feic" | "téigh" if is_past => ("ní", Mutation::Len1),
        "faigh" if is_past => ("ní", Mutation::Ecl1),
        "faigh" if matches!(tense, VerbTense::Fut | VerbTense::Cond) => ("ní", Mutation::Ecl1),
        "tar" | "clois" | "cluin" if is_past && is_auto => ("níor", Mutation::Len1),
        _ if is_past && is_auto => ("níor", Mutation::Nil),
        _ if is_past => ("níor", Mutation::Len1),
        _ => ("ní", Mutation::Len1),
    }
}

// ---------------------------------------------------------------------------
// Relative independent forms (Python v2 addition)
// ---------------------------------------------------------------------------
// For regular verbs only. Present/PresCont: strip -nn → -s.
// Future: strip -fidh → -feas, or -idh → -s.

fn generate_rel_indep(v: &mut Verb) {
    // Present habitual
    if let Some(base) = v.pres_cont.indep.base.first() {
        let val = &base.value;
        if val.ends_with("nn") {
            let rel = format!("{}s", &val[..val.len() - 2]);
            v.pres_cont.rel_indep.set(VerbPerson::Base, &rel);
        }
    }

    // Future
    if let Some(base) = v.fut.indep.base.first() {
        let val = &base.value;
        if val.ends_with("fidh") {
            let rel = format!("{}feas", &val[..val.len() - 4]);
            v.fut.rel_indep.set(VerbPerson::Base, &rel);
        } else if val.ends_with("idh") {
            let rel = format!("{}s", &val[..val.len() - 3]);
            v.fut.rel_indep.set(VerbPerson::Base, &rel);
        }
    }
}

// ---------------------------------------------------------------------------
// 1st conjugation
// ---------------------------------------------------------------------------

/// Polysyllabic 1st-conj verbs with long-vowel endings depalatalise (broaden).
fn is_syncopating_first(lemma: &str) -> bool {
    let l = lemma.to_lowercase();
    if l.ends_with("iomáin") { return false; }
    if l.ends_with("dháil") { return false; }
    opers::polysyllabic(lemma) &&
    ["áil", "eáil", "óil", "úil", "áin", "eáin", "óin"]
        .iter()
        .any(|s| l.ends_with(s))
}

/// Additional 1st-conj verbs that need opers::broaden() applied as stem.
/// Only includes patterns with zero known false positives in BuNaMo.
fn needs_broadening_first(lemma: &str) -> bool {
    let l = lemma.to_lowercase();

    if l.ends_with("laic") { return true; }
    if l.ends_with("achaid") { return true; }
    if l.ends_with("ailc") && !l.ends_with("mailc") { return true; }
    if l.ends_with("aich") { return true; }
    if l.ends_with("oim") { return true; }
    if l.ends_with("maidhm") || l.ends_with("haidhm") { return true; }
    if l.ends_with("úraic") { return true; }
    if l.ends_with("onnail") { return true; }
    if l.ends_with("ochais") { return true; }
    if l.ends_with("ómhais") { return true; }

    if opers::polysyllabic(lemma) {
        if l.ends_with("aill") { return true; }
        if l.ends_with("úir") || l.ends_with("óir") { return true; }
        if l.ends_with("aig") { return true; }
        if l.ends_with("úin") { return true; }
        if l.ends_with("maisc") || l.ends_with("laisc") || l.ends_with("misc") {
            return true;
        }
        if l.ends_with("airg") && !l.ends_with("thairg") { return true; }
    }

    if !opers::polysyllabic(lemma) {
        // Long-vowel endings where a vowel precedes the suffix
        for suf in &["áil", "eáil", "óil", "úil", "úin", "úir", "ain", "aim"] {
            if l.ends_with(suf) {
                if *suf == "ain" && (l.ends_with("áin") || l.ends_with("eáin")) {
                    continue;
                }
                let before = &l[..l.len() - suf.len()];
                if before.chars().last().map_or(false, |c| opers::VOWELS.contains(c)) {
                    return true;
                }
            }
        }
        // Monosyllabic long-vowel endings with no vowel-before requirement
        // (only one verb each in BuNaMo, all confirmed broadening)
        if l.ends_with("óin") || l.ends_with("úim") { return true; }
    }

    false
}

/// 1st-conj verbs needing opers::syncope() — full vowel-cluster removal,
/// not just i-removal (broaden). For these verbs syncope gives the correct
/// stem for ALL tenses (future, conditional, past synthetic).
fn needs_syncope_first(lemma: &str) -> bool {
    let l = lemma.to_lowercase();

    if !opers::polysyllabic(lemma) { return false; }

    if l.ends_with("dhair") || l.ends_with("dhar") { return true; }
    if l.ends_with("mhain") { return true; }
    if l.ends_with("ghail") { return true; }
    if l.ends_with("abhac") { return true; }
    if l.ends_with("ighid") { return true; }
    if l.ends_with("eighil") || l.ends_with("uighill") { return true; }

    false
}

/// Verbs with deep syncopation where per-verb stem data is needed to match
/// the Caighdeán/BuNaMo paradigms. Each verb may use a different stem for
/// different slot groups (syncopated, broadened, or lemma).
///
/// Columns: (suffix, syn, ss, t, pt, f, c)
/// - syn:  syncopated stem (present base, past auto, default synthetic)
/// - ss:   synth stem override for non-auto synthetic forms; None = syn
/// - t:    pastcont t-suffix stem (sg2, auto)
/// - pt:   prescont/imper/subj auto t-suffix stem (= t except achair)
/// - f:    future base (+ "f" → future stem)
/// - c:    conditional base (+ "f" → cond stem; = f except adhain/athadhain)
///
/// deighil: f/c use lemma (gnag's deighilfidh, attested on Wiktionary).
const DEEP_SYNCOPE_VERBS: &[(&str, &str, Option<&str>, &str, &str, &str, &str)] = &[
    //                syn        ss           t           pt          f           c
    ("achair",    "achr",    None,         "achar",    "achair",  "achar",    "achar"),
    ("adhain",    "adhn",    None,         "adhn",     "adhn",    "adhan",    "adhn"),
    ("athadhain", "athadhn", None,         "athadhn",  "athadhn", "athadhan", "athadhn"),
    ("deighil",   "deighl",  None,         "deighl",   "deighl",  "deighil",  "deighil"),
    ("déadail",   "déadl",   None,         "déadla",   "déadla",  "déadla",   "déadla"),
    ("diongaibh", "diongbh", None,         "diongbha", "diongbha","diongbha", "diongbha"),
    ("ionnail",   "ionnal",  Some("ionl"), "ionnal",   "ionnal",  "ionnal",   "ionnal"),
    ("tiomain",   "tiomn",   None,         "tioman",   "tioman",  "tioman",   "tioman"),
];

struct DeepSyncopeResult {
    stem: String,
    synth_stem: Option<String>,
    t_stem: String,
    pres_t_stem: String,
    f_base: String,
    c_base: String,
}

fn lookup_deep_syncope(lemma: &str) -> Option<DeepSyncopeResult> {
    let l = lemma.to_lowercase();
    for &(suffix, syn_end, ss_end, t_end, pt_end, f_end, c_end) in DEEP_SYNCOPE_VERBS {
        if l.ends_with(suffix) {
            let prefix = &lemma[..lemma.len() - suffix.len()];
            if prefix.is_empty()
                || prefix.ends_with(|c: char| c == '-' || c == 'h' || c == 'n' || c == 't' || c == 'd')
            {
                return Some(DeepSyncopeResult {
                    stem: format!("{}{}", prefix, syn_end),
                    synth_stem: ss_end.map(|s| format!("{}{}", prefix, s)),
                    t_stem: format!("{}{}", prefix, t_end),
                    pres_t_stem: format!("{}{}", prefix, pt_end),
                    f_base: format!("{}{}", prefix, f_end),
                    c_base: format!("{}{}", prefix, c_end),
                });
            }
        }
    }
    None
}

/// Monosyllabic -igh verbs whose root ends in a long vowel or diphthong.
/// These strip -igh for conjugation: báigh→bá, léigh→lé, truaigh→trua.
/// Excludes short-vowel/consonant roots (suigh, nigh) which need í-insertion.
fn is_igh_vowel_root(lemma: &str) -> bool {
    let l = lemma.to_lowercase();
    if !l.ends_with("igh") { return false; }
    let root = &l[..l.len() - 3];
    if root.is_empty() { return false; }
    let last = root.chars().last().unwrap();
    if "áéóú".contains(last) { return true; }
    root.ends_with("ua") || root.ends_with("ia")
        || root.ends_with("ao") || root.ends_with("eo")
        || root.ends_with("ae")
}

/// Suffix joining for vowel-final stems (monosyllabic -igh verbs).
/// Handles vowel cluster merging at the stem-suffix boundary:
/// - é-root + "adh" → "léadh" (éa is a valid cluster)
/// - é-root + "aim" → "léim" (use slender; éai would be 3 vowels)
/// - broad root + "adh" → "dódh" (strip redundant 'a')
/// - broad root + "aim" → "dóim" (strip 'a', same as slender)
fn suf_v(root: &str, broad: &str, slender: &str) -> String {
    if broad == "a" || broad == "e" {
        return root.to_string();
    }
    let last = root.chars().last().unwrap_or(' ');
    if last == 'é' {
        let second = broad.chars().nth(1).unwrap_or(' ');
        if broad.starts_with('a') && opers::VOWELS.contains(second) {
            format!("{}{}", root, slender)
        } else {
            format!("{}{}", root, broad)
        }
    } else {
        if broad.starts_with('a') {
            format!("{}{}", root, &broad['a'.len_utf8()..])
        } else {
            format!("{}{}", root, broad)
        }
    }
}

fn is_igh_short_root(lemma: &str) -> bool {
    let l = lemma.to_lowercase();
    if !l.ends_with("igh") { return false; }
    if l == "saigh" { return false; }
    let root = &l[..l.len() - 3];
    if root.is_empty() { return false; }
    let last = root.chars().last().unwrap();
    if "aeiou".contains(last) { return true; }
    if !root.chars().any(|c| opers::VOWELS.contains(c)) { return true; }
    root.ends_with("fh")
}

fn suf_i(stem: &str, broad: &str, slender: &str) -> String {
    if broad == "a" || broad == "e" {
        return stem.to_string();
    }
    if slender.starts_with('i') {
        format!("{}{}", stem, &slender['i'.len_utf8()..])
    } else {
        format!("{}o{}", stem, &broad['a'.len_utf8()..])
    }
}

fn prefix_all_forms(pf: &mut PersonForms, prefix: &str) {
    for slot in [
        &mut pf.base, &mut pf.sg1, &mut pf.sg2, &mut pf.sg3,
        &mut pf.pl1, &mut pf.pl2, &mut pf.pl3, &mut pf.auto,
    ] {
        for form in slot.iter_mut() {
            form.value = format!("{}{}", prefix, form.value);
        }
    }
}

fn apply_d_prefix(v: &mut Verb) {
    // Past: d' on all except auto
    for pf in [&mut v.past.indep, &mut v.past.dep] {
        for slot in [
            &mut pf.base, &mut pf.sg1, &mut pf.sg2, &mut pf.sg3,
            &mut pf.pl1, &mut pf.pl2, &mut pf.pl3,
        ] {
            for form in slot.iter_mut() {
                form.value = format!("d'{}", form.value);
            }
        }
    }
    // PastCont and Cond: d' on all forms including auto
    prefix_all_forms(&mut v.past_cont.indep, "d'");
    prefix_all_forms(&mut v.past_cont.dep, "d'");
    prefix_all_forms(&mut v.cond.indep, "d'");
    prefix_all_forms(&mut v.cond.dep, "d'");
}

fn lookup_suppletive_first(lemma: &str) -> Option<&'static str> {
    let l = lemma.to_lowercase();
    SUPPLETIVE_FIRST.iter()
        .find(|&&(lem, _)| l == lem)
        .map(|&(_, root)| root)
}

fn conjugate_first(lemma: &str) -> Verb {
    let mut v = Verb::empty(VerbConjugationClass::First);

    if is_igh_vowel_root(lemma) {
        return conjugate_first_igh(&mut v, lemma);
    }
    let l_lower = lemma.to_lowercase();
    if l_lower.ends_with("ígh") || is_igh_short_root(lemma) {
        return conjugate_first_igh_short(&mut v, lemma);
    }

    let suppletive = lookup_suppletive_first(lemma);
    let effective_lemma = suppletive.unwrap_or(lemma);

    let ds_result = lookup_deep_syncope(effective_lemma);

    // (stem, slender, t_stem, pres_t_stem, f_base, c_base)
    // stem: syncopated/broadened, for present analytic + past auto
    // t_stem: for pastcont sg2/auto
    // pres_t_stem: for prescont/imper/subj auto (= t_stem except achair)
    // f_base: for future (f_base + "f" → f_stem)
    // c_base: for conditional (c_base + "f" → c_stem; = f_base except adhain/athadhain)
    let el = effective_lemma;
    let (stem, sl, t_stem, pres_t_stem, f_base, c_base) = if l_lower.ends_with("igh") && opers::polysyllabic(lemma) {
        let root = el[..el.len() - "igh".len()].to_string();
        let root_sl = opers::is_slender(&root);
        (root.clone(), root_sl, root.clone(), root.clone(), root.clone(), root)
    } else if is_syncopating_first(el) {
        let broad = opers::broaden(el);
        let broad_sl = opers::is_slender(&broad);
        let ll = el.to_lowercase();
        let uses_lemma_t = (ll.ends_with("áil") || ll.ends_with("eáil"))
            && ll != "annáil";
        let ts = if uses_lemma_t { el.to_string() } else { broad.clone() };
        (broad.clone(), broad_sl, ts.clone(), ts, broad.clone(), broad)
    } else if let Some(ref ds) = ds_result {
        let syn_sl = opers::is_slender(&ds.stem);
        (ds.stem.clone(), syn_sl, ds.t_stem.clone(), ds.pres_t_stem.clone(), ds.f_base.clone(), ds.c_base.clone())
    } else if needs_syncope_first(el) {
        let syn = opers::syncope(el);
        let syn_sl = opers::is_slender(&syn);
        (syn.clone(), syn_sl, syn.clone(), syn.clone(), syn.clone(), syn)
    } else if needs_broadening_first(el) {
        let broad = opers::broaden(el);
        let broad_sl = opers::is_slender(&broad);
        let ell = el.to_lowercase();
        let uses_lemma_t = (ell.ends_with("áil") || ell.ends_with("eáil"))
            && ell != "annáil";
        let ts = if uses_lemma_t { el.to_string() } else { broad.clone() };
        (broad.clone(), broad_sl, ts.clone(), ts, broad.clone(), broad)
    } else {
        let s = opers::is_slender(el);
        (el.to_string(), s, el.to_string(), el.to_string(), el.to_string(), el.to_string())
    };

    // For deep-syncope verbs with an alternate synthetic stem (e.g. ionnail→ionl),
    // use it for non-auto, non-t-suffix synthetic forms
    let synth = ds_result.as_ref().and_then(|ds| ds.synth_stem.as_ref());
    let synth_sl = synth.map(|s| opers::is_slender(s));

    // synth_s/synth_sl: alternate stem for non-auto synthetic forms (e.g. ionnail→ionl)
    let (synth_s, ss_sl) = match synth {
        Some(s) => (s.as_str(), synth_sl.unwrap()),
        None => (stem.as_str(), sl),
    };

    // Past: base = lemma, synthetic Pl1/Pl3 use synth stem, Auto uses main stem
    v.past.set_both(VerbPerson::Base, lemma);
    v.past.set_both(VerbPerson::Pl1,  &suf(synth_s, ss_sl, "amar", "eamar"));
    v.past.set_both(VerbPerson::Pl3,  &suf(synth_s, ss_sl, "adar", "eadar"));
    v.past.set_both(VerbPerson::Auto, &suf(&stem, sl, "adh",  "eadh"));

    // Past habitual (PastCont)
    v.past_cont.set_both(VerbPerson::Base, &suf(synth_s, ss_sl, "adh",   "eadh"));
    v.past_cont.set_both(VerbPerson::Sg1,  &suf(synth_s, ss_sl, "ainn",  "inn"));
    v.past_cont.set_both(VerbPerson::Sg2,  &suf_t(&t_stem, "tá",    "teá"));
    v.past_cont.set_both(VerbPerson::Pl1,  &suf(synth_s, ss_sl, "aimis", "imis"));
    v.past_cont.set_both(VerbPerson::Pl3,  &suf(synth_s, ss_sl, "aidís", "idís"));
    v.past_cont.set_both(VerbPerson::Auto, &suf_t(&t_stem, "taí",   "tí"));

    // Present habitual (PresCont)
    v.pres_cont.set_both(VerbPerson::Base, &suf(synth_s, ss_sl, "ann",   "eann"));
    v.pres_cont.set_both(VerbPerson::Sg1,  &suf(synth_s, ss_sl, "aim",   "im"));
    v.pres_cont.set_both(VerbPerson::Pl1,  &suf(synth_s, ss_sl, "aimid", "imid"));
    v.pres_cont.set_both(VerbPerson::Auto, &suf_t(&pres_t_stem, "tar",   "tear"));

    let f_stem = if f_base.ends_with('f') {
        f_base.clone()
    } else {
        format!("{}f", f_base)
    };
    v.fut.set_both(VerbPerson::Base, &suf(&f_stem, sl, "aidh",  "idh"));
    v.fut.set_both(VerbPerson::Pl1,  &suf(&f_stem, sl, "aimid", "imid"));
    v.fut.set_both(VerbPerson::Auto, &suf(&f_stem, sl, "ar",    "ear"));

    let c_stem = if c_base.ends_with('f') {
        c_base.clone()
    } else {
        format!("{}f", c_base)
    };
    v.cond.set_both(VerbPerson::Base, &suf(&c_stem, sl, "adh",   "eadh"));
    v.cond.set_both(VerbPerson::Sg1,  &suf(&c_stem, sl, "ainn",  "inn"));
    v.cond.set_both(VerbPerson::Sg2,  &suf(&c_stem, sl, "á",     "eá"));
    v.cond.set_both(VerbPerson::Pl1,  &suf(&c_stem, sl, "aimis", "imis"));
    v.cond.set_both(VerbPerson::Pl3,  &suf(&c_stem, sl, "aidís", "idís"));
    v.cond.set_both(VerbPerson::Auto, &suf(&c_stem, sl, "aí",    "í"));

    // Imperative
    v.imper.set(VerbPerson::Base, &suf(synth_s, ss_sl, "adh",   "eadh"));
    v.imper.set(VerbPerson::Sg1,  &suf(synth_s, ss_sl, "aim",   "im"));
    v.imper.set(VerbPerson::Sg2,  lemma);
    v.imper.set(VerbPerson::Pl1,  &suf(synth_s, ss_sl, "aimis", "imis"));
    v.imper.set(VerbPerson::Pl2,  &suf(synth_s, ss_sl, "aigí",  "igí"));
    v.imper.set(VerbPerson::Pl3,  &suf(synth_s, ss_sl, "aidís", "idís"));
    v.imper.set(VerbPerson::Auto, &suf_t(&pres_t_stem, "tar",   "tear"));

    // Subjunctive
    v.subj.set(VerbPerson::Base, &suf(synth_s, ss_sl, "a",     "e"));
    v.subj.set(VerbPerson::Pl1,  &suf(synth_s, ss_sl, "aimid", "imid"));
    v.subj.set(VerbPerson::Auto, &suf_t(&pres_t_stem, "tar",   "tear"));

    generate_rel_indep(&mut v);
    v
}

/// Conjugate monosyllabic -igh verbs with vowel-final roots.
/// Root = lemma minus -igh; t-stem = lemma minus -gh (slender, for t-suffixes).
/// Future stem: é-roots use slender stem + f; broad roots use root + f.
fn conjugate_first_igh(v: &mut Verb, lemma: &str) -> Verb {
    let root = &lemma[..lemma.len() - "igh".len()];
    let root_is_ia = root.to_lowercase().ends_with("ia");
    let (t_stem, t_sl) = if root_is_ia {
        (root.to_string(), false)
    } else {
        (lemma[..lemma.len() - "gh".len()].to_string(), true)
    };

    let root_is_é = root.chars().last() == Some('é');

    let f_stem = if root_is_é {
        format!("{}f", t_stem)
    } else {
        format!("{}f", root)
    };
    let f_sl = opers::is_slender(&f_stem)
        && !root.to_lowercase().ends_with("ae");

    v.past.set_both(VerbPerson::Base, lemma);
    v.past.set_both(VerbPerson::Pl1,  &suf_v(root, "amar", "eamar"));
    v.past.set_both(VerbPerson::Pl3,  &suf_v(root, "adar", "eadar"));
    v.past.set_both(VerbPerson::Auto, &suf_v(root, "adh",  "eadh"));

    v.past_cont.set_both(VerbPerson::Base, &suf_v(root, "adh",   "eadh"));
    v.past_cont.set_both(VerbPerson::Sg1,  &suf_v(root, "ainn",  "inn"));
    v.past_cont.set_both(VerbPerson::Sg2,  &suf(&t_stem, t_sl, "tá", "teá"));
    v.past_cont.set_both(VerbPerson::Pl1,  &suf_v(root, "aimis", "imis"));
    v.past_cont.set_both(VerbPerson::Pl3,  &suf_v(root, "aidís", "idís"));
    v.past_cont.set_both(VerbPerson::Auto, &suf(&t_stem, t_sl, "taí", "tí"));

    v.pres_cont.set_both(VerbPerson::Base, &suf_v(root, "ann",   "eann"));
    v.pres_cont.set_both(VerbPerson::Sg1,  &suf_v(root, "aim",   "im"));
    v.pres_cont.set_both(VerbPerson::Pl1,  &suf_v(root, "aimid", "imid"));
    v.pres_cont.set_both(VerbPerson::Auto, &suf(&t_stem, t_sl, "tar", "tear"));

    v.fut.set_both(VerbPerson::Base, &suf(&f_stem, f_sl, "aidh",  "idh"));
    v.fut.set_both(VerbPerson::Pl1,  &suf(&f_stem, f_sl, "aimid", "imid"));
    v.fut.set_both(VerbPerson::Auto, &suf(&f_stem, f_sl, "ar",    "ear"));

    v.cond.set_both(VerbPerson::Base, &suf(&f_stem, f_sl, "adh",   "eadh"));
    v.cond.set_both(VerbPerson::Sg1,  &suf(&f_stem, f_sl, "ainn",  "inn"));
    v.cond.set_both(VerbPerson::Sg2,  &suf(&f_stem, f_sl, "á",     "eá"));
    v.cond.set_both(VerbPerson::Pl1,  &suf(&f_stem, f_sl, "aimis", "imis"));
    v.cond.set_both(VerbPerson::Pl3,  &suf(&f_stem, f_sl, "aidís", "idís"));
    v.cond.set_both(VerbPerson::Auto, &suf(&f_stem, f_sl, "aí",    "í"));

    v.imper.set(VerbPerson::Base, &suf_v(root, "adh",   "eadh"));
    v.imper.set(VerbPerson::Sg1,  &suf_v(root, "aim",   "im"));
    v.imper.set(VerbPerson::Sg2,  lemma);
    v.imper.set(VerbPerson::Pl1,  &suf_v(root, "aimis", "imis"));
    v.imper.set(VerbPerson::Pl2,  &suf_v(root, "aigí",  "igí"));
    v.imper.set(VerbPerson::Pl3,  &suf_v(root, "aidís", "idís"));
    v.imper.set(VerbPerson::Auto, &suf(&t_stem, t_sl, "tar", "tear"));

    v.subj.set(VerbPerson::Base, &suf_v(root, "a",     "e"));
    v.subj.set(VerbPerson::Pl1,  &suf_v(root, "aimid", "imid"));
    v.subj.set(VerbPerson::Auto, &suf(&t_stem, t_sl, "tar", "tear"));

    generate_rel_indep(v);
    v.clone()
}

fn conjugate_first_igh_short(v: &mut Verb, lemma: &str) -> Verb {
    let strip = if lemma.to_lowercase().ends_with("ígh") { "ígh".len() } else { "igh".len() };
    let root = &lemma[..lemma.len() - strip];
    let i_stem = format!("{}í", root);
    let t_stem = lemma[..lemma.len() - "gh".len()].to_string();
    let f_stem = format!("{}f", i_stem);
    let f_sl = opers::is_slender(&f_stem);

    v.past.set_both(VerbPerson::Base, lemma);
    v.past.set_both(VerbPerson::Pl1,  &suf_i(&i_stem, "amar", "eamar"));
    v.past.set_both(VerbPerson::Pl3,  &suf_i(&i_stem, "adar", "eadar"));
    v.past.set_both(VerbPerson::Auto, &suf_i(&i_stem, "adh",  "eadh"));

    v.past_cont.set_both(VerbPerson::Base, &suf_i(&i_stem, "adh",   "eadh"));
    v.past_cont.set_both(VerbPerson::Sg1,  &suf_i(&i_stem, "ainn",  "inn"));
    v.past_cont.set_both(VerbPerson::Sg2,  &suf(&t_stem, true, "tá", "teá"));
    v.past_cont.set_both(VerbPerson::Pl1,  &suf_i(&i_stem, "aimis", "imis"));
    v.past_cont.set_both(VerbPerson::Pl3,  &suf_i(&i_stem, "aidís", "idís"));
    v.past_cont.set_both(VerbPerson::Auto, &suf(&t_stem, true, "taí", "tí"));

    v.pres_cont.set_both(VerbPerson::Base, &suf_i(&i_stem, "ann",   "eann"));
    v.pres_cont.set_both(VerbPerson::Sg1,  &suf_i(&i_stem, "aim",   "im"));
    v.pres_cont.set_both(VerbPerson::Pl1,  &suf_i(&i_stem, "aimid", "imid"));
    v.pres_cont.set_both(VerbPerson::Auto, &suf(&t_stem, true, "tar", "tear"));

    v.fut.set_both(VerbPerson::Base, &suf(&f_stem, f_sl, "aidh",  "idh"));
    v.fut.set_both(VerbPerson::Pl1,  &suf(&f_stem, f_sl, "aimid", "imid"));
    v.fut.set_both(VerbPerson::Auto, &suf(&f_stem, f_sl, "ar",    "ear"));

    v.cond.set_both(VerbPerson::Base, &suf(&f_stem, f_sl, "adh",   "eadh"));
    v.cond.set_both(VerbPerson::Sg1,  &suf(&f_stem, f_sl, "ainn",  "inn"));
    v.cond.set_both(VerbPerson::Sg2,  &suf(&f_stem, f_sl, "á",     "eá"));
    v.cond.set_both(VerbPerson::Pl1,  &suf(&f_stem, f_sl, "aimis", "imis"));
    v.cond.set_both(VerbPerson::Pl3,  &suf(&f_stem, f_sl, "aidís", "idís"));
    v.cond.set_both(VerbPerson::Auto, &suf(&f_stem, f_sl, "aí",    "í"));

    v.imper.set(VerbPerson::Base, &suf_i(&i_stem, "adh",   "eadh"));
    v.imper.set(VerbPerson::Sg1,  &suf_i(&i_stem, "aim",   "im"));
    v.imper.set(VerbPerson::Sg2,  lemma);
    v.imper.set(VerbPerson::Pl1,  &suf_i(&i_stem, "aimis", "imis"));
    v.imper.set(VerbPerson::Pl2,  &suf_i(&i_stem, "aigí",  "igí"));
    v.imper.set(VerbPerson::Pl3,  &suf_i(&i_stem, "aidís", "idís"));
    v.imper.set(VerbPerson::Auto, &suf(&t_stem, true, "tar", "tear"));

    v.subj.set(VerbPerson::Base, &suf_i(&i_stem, "a",     "e"));
    v.subj.set(VerbPerson::Pl1,  &suf_i(&i_stem, "aimid", "imid"));
    v.subj.set(VerbPerson::Auto, &suf(&t_stem, true, "tar", "tear"));

    generate_rel_indep(v);
    v.clone()
}

/// Append a broad or slender suffix to a stem.
fn suf(stem: &str, slender: bool, broad: &str, sl: &str) -> String {
    format!("{}{}", stem, if slender { sl } else { broad })
}

/// Append a t-initial suffix (tá/teá, taí/tí, tar/tear) with consonant cluster handling.
/// Stems ending in -th strip the h (merging with suffix t); stems already ending
/// in -t absorb the suffix's leading t to avoid doubling.
fn suf_t(stem: &str, broad: &str, slender: &str) -> String {
    let sl = opers::is_slender(stem);
    let suffix = if sl { slender } else { broad };

    let effective_stem = if stem.ends_with("th") {
        &stem[..stem.len() - 'h'.len_utf8()]
    } else {
        stem
    };
    if effective_stem.ends_with('t') && suffix.starts_with('t') {
        format!("{}{}", effective_stem, &suffix['t'.len_utf8()..])
    } else {
        format!("{}{}", effective_stem, suffix)
    }
}

// ---------------------------------------------------------------------------
// 2nd conjugation
// ---------------------------------------------------------------------------

fn is_non_syncopating_second(lemma: &str) -> bool {
    let l = lemma.to_lowercase();
    if !opers::polysyllabic(lemma) { return true; }
    if l.ends_with("ing") { return true; }
    if l.ends_with("aim") { return true; }
    if l.ends_with("aithris") { return true; }
    if l.ends_with("úraic") { return true; }
    false
}

fn needs_broadening_second(lemma: &str) -> bool {
    let l = lemma.to_lowercase();
    l.ends_with("freastail") || l.ends_with("gogail") || l.ends_with("taistil")
}

fn root_2nd(lemma: &str) -> String {
    let l = lemma.to_lowercase();
    if l.ends_with("ígh") {
        lemma[..lemma.len() - "ígh".len()].to_string()
    } else if lemma.ends_with("igh") {
        lemma[..lemma.len() - "igh".len()].to_string()
    } else if is_non_syncopating_second(lemma) {
        lemma.to_string()
    } else if needs_broadening_second(lemma) {
        opers::broaden(lemma)
    } else {
        let root = opers::syncope(lemma);
        if l.ends_with("innill") {
            root.replacen("innl", "inl", 1)
        } else {
            root
        }
    }
}

/// The bare root stripped of any trailing vowels — used for future/conditional stem.
fn bare_root_2nd(lemma: &str) -> String {
    let l = lemma.to_lowercase();
    if let Some(&(_, root)) = ROOT_OVERRIDE_SECOND.iter().find(|&&(lem, _)| l == lem) {
        return root.to_string();
    }
    let root = root_2nd(lemma);
    if lemma.ends_with("aigh") {
        let r = &root[..root.len() - 'a'.len_utf8()];
        r.to_string()
    } else if lemma.ends_with("igh") {
        root
    } else {
        root
    }
}

fn conjugate_second(lemma: &str) -> Verb {
    let mut v = Verb::empty(VerbConjugationClass::Second);

    let bare = bare_root_2nd(lemma);
    let l_lower = lemma.to_lowercase();
    let root_is_slender = SLENDER_OVERRIDE_SECOND.iter().any(|&w| l_lower == w)
        || opers::is_slender_i(&bare);

    // Present stem: bare + aí/í (always ends slender due to í)
    let pres_stem = if root_is_slender {
        format!("{}í", bare)
    } else {
        format!("{}aí", bare)
    };

    // Future stem: bare + ó/eo (always ends broad due to ó/o)
    let fut_stem = if root_is_slender {
        format!("{}eo", bare)
    } else {
        format!("{}ó", bare)
    };

    // Past: base = lemma, synthetic forms use present stem
    v.past.set_both(VerbPerson::Base, lemma);
    v.past.set_both(VerbPerson::Pl1,  &format!("{}omar", pres_stem));
    v.past.set_both(VerbPerson::Pl3,  &format!("{}odar", pres_stem));
    v.past.set_both(VerbPerson::Auto, &format!("{}odh",  pres_stem));

    // Past habitual
    v.past_cont.set_both(VerbPerson::Base, &format!("{}odh",  pres_stem));
    v.past_cont.set_both(VerbPerson::Sg1,  &format!("{}nn",   pres_stem));
    v.past_cont.set_both(VerbPerson::Sg2,  &format!("{}teá",  pres_stem));
    v.past_cont.set_both(VerbPerson::Pl1,  &format!("{}mis",  pres_stem));
    v.past_cont.set_both(VerbPerson::Pl3,  &format!("{}dís",  pres_stem));
    v.past_cont.set_both(VerbPerson::Auto, &format!("{}tí",   pres_stem));

    // Present habitual
    v.pres_cont.set_both(VerbPerson::Base, &format!("{}onn",  pres_stem));
    v.pres_cont.set_both(VerbPerson::Sg1,  &format!("{}m",    pres_stem));
    v.pres_cont.set_both(VerbPerson::Pl1,  &format!("{}mid",  pres_stem));
    v.pres_cont.set_both(VerbPerson::Auto, &format!("{}tear", pres_stem));

    // Future
    v.fut.set_both(VerbPerson::Base, &format!("{}idh",  fut_stem));
    v.fut.set_both(VerbPerson::Pl1,  &format!("{}imid", fut_stem));
    v.fut.set_both(VerbPerson::Auto, &format!("{}far",  fut_stem));

    // Conditional
    v.cond.set_both(VerbPerson::Base, &format!("{}dh",   fut_stem));
    v.cond.set_both(VerbPerson::Sg1,  &format!("{}inn",  fut_stem));
    v.cond.set_both(VerbPerson::Sg2,  &format!("{}fá",   fut_stem));
    v.cond.set_both(VerbPerson::Pl1,  &format!("{}imis", fut_stem));
    v.cond.set_both(VerbPerson::Pl3,  &format!("{}idís", fut_stem));
    v.cond.set_both(VerbPerson::Auto, &format!("{}faí",  fut_stem));

    // Imperative
    v.imper.set(VerbPerson::Base, &format!("{}odh",  pres_stem));
    v.imper.set(VerbPerson::Sg1,  &format!("{}m",    pres_stem));
    v.imper.set(VerbPerson::Sg2,  lemma);
    v.imper.set(VerbPerson::Pl1,  &format!("{}mis",  pres_stem));
    v.imper.set(VerbPerson::Pl2,  &format!("{}gí",   pres_stem));
    v.imper.set(VerbPerson::Pl3,  &format!("{}dís",  pres_stem));
    v.imper.set(VerbPerson::Auto, &format!("{}tear", pres_stem));

    // Subjunctive
    v.subj.set(VerbPerson::Base, &pres_stem);
    v.subj.set(VerbPerson::Pl1,  &format!("{}mid",  pres_stem));
    v.subj.set(VerbPerson::Auto, &format!("{}tear", pres_stem));

    generate_rel_indep(&mut v);
    v
}

// ---------------------------------------------------------------------------
// Irregular verbs — embedded BuNaMo XML, parsed once via Lazy
// ---------------------------------------------------------------------------

static IRREGULAR_ABAIR:   Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/abair_verb.xml")));
static IRREGULAR_BEIR:    Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/beir_verb.xml")));
static IRREGULAR_BI:      Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/bí_verb.xml")));
static IRREGULAR_CLOIS:   Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/clois_verb.xml")));
static IRREGULAR_CLUIN:   Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/cluin_verb.xml")));
static IRREGULAR_DEAN:    Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/déan_verb.xml")));
static IRREGULAR_FAIGH:   Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/faigh_verb.xml")));
static IRREGULAR_FEIC:    Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/feic_verb.xml")));
static IRREGULAR_ITH:     Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/ith_verb.xml")));
static IRREGULAR_TABHAIR: Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/tabhair_verb.xml")));
static IRREGULAR_TAR:     Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/tar_verb.xml")));
static IRREGULAR_TEIGH:   Lazy<Verb> = Lazy::new(|| Verb::from_xml(include_str!("../../data/verb/téigh_verb.xml")));

fn conjugate_irregular(lemma: &str) -> Verb {
    let v = match lemma {
        "abair"   => &*IRREGULAR_ABAIR,
        "beir"    => &*IRREGULAR_BEIR,
        "bí"      => &*IRREGULAR_BI,
        "clois"   => &*IRREGULAR_CLOIS,
        "cluin"   => &*IRREGULAR_CLUIN,
        "déan"    => &*IRREGULAR_DEAN,
        "faigh"   => &*IRREGULAR_FAIGH,
        "feic"    => &*IRREGULAR_FEIC,
        "ith"     => &*IRREGULAR_ITH,
        "tabhair" => &*IRREGULAR_TABHAIR,
        "tar"     => &*IRREGULAR_TAR,
        "téigh"   => &*IRREGULAR_TEIGH,
        _ => {
            let mut v = conjugate_first(lemma);
            v.conjugation = VerbConjugationClass::Irregular;
            return v;
        }
    };
    v.clone()
}

// ---------------------------------------------------------------------------
// BuNaMo XML parser for verb data
// ---------------------------------------------------------------------------

fn parse_person(s: &str) -> Option<VerbPerson> {
    match s {
        "Base" => Some(VerbPerson::Base),
        "Sg1"  => Some(VerbPerson::Sg1),
        "Sg2"  => Some(VerbPerson::Sg2),
        "Sg3"  => Some(VerbPerson::Sg3),
        "Pl1"  => Some(VerbPerson::Pl1),
        "Pl2"  => Some(VerbPerson::Pl2),
        "Pl3"  => Some(VerbPerson::Pl3),
        "Auto" => Some(VerbPerson::Auto),
        _ => None,
    }
}

fn xml_attrs(e: &quick_xml::events::BytesStart<'_>) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        map.insert(key, val);
    }
    map
}

// ---------------------------------------------------------------------------
// Verbal adjective generation (gnag rules)
// ---------------------------------------------------------------------------

// 1st-conj roots needing VA broadening that the conjugation predicates miss.
// Includes both base and lenited forms so compounds match (e.g. aischuir).
const VA_BROADEN_ROOTS: &[(&str, &str)] = &[
    ("cuir", "cur"),   ("chuir", "chur"),
    ("buair", "buar"),
    ("mair", "mar"),   ("mhair", "mhar"),
    ("coir", "cor"),   ("choir", "chor"),
    ("scoir", "scor"), ("schoir", "schor"),
    ("doir", "dor"),   ("dhoir", "dhor"),
    ("goin", "gon"),   ("ghoin", "ghon"),
    ("broim", "brom"),
    ("uaim", "uam"),
    ("foirceann", "foircean"),
    ("tiomain", "tioman"), ("thiomain", "thioman"),
    ("cinnir", "cinnear"),
    ("cónaisc", "cónasc"),
    ("déadail", "déadal"),
];

/// Irregular verbal adjective forms that can't be derived by rule.
/// Sorted longest-suffix-first: if suffix A is a suffix of B, B must precede A.
const VA_OVERRIDES: &[(&str, &str)] = &[
    // 2nd conj: -thaigh/-thigh (VA strips past the -th)
    ("lúthaigh", "lúite"),
    ("dhathaigh", "dhaite"),
    ("dlaoithigh", "dlaoithe"),
    ("dathaigh", "daite"),
    ("táthaigh", "táite"),
    // 2nd conj: irregular stems
    ("feallmharaigh", "feallmharfa"),
    ("aiceannaigh", "aiceanta"),
    ("míshásaigh", "míshásta"),
    ("miadhaigh", "miadhuighthe"),
    ("thadhaill", "thadhallta"),
    ("tairngir", "tairngirithe"),
    ("deachair", "deachraithe"),
    ("freastail", "freastalta"),
    ("líomhain", "líomhainte"),
    ("tuargain", "tuargainte"),
    ("stoithin", "stoithinte"),
    ("tadhaill", "tadhallta"),
    ("tafainn", "tafannta"),
    ("taistil", "taistealta"),
    ("cadhail", "caidhilte"),
    ("codail", "codalta"),
    ("innill", "innealta"),
    ("locair", "locraithe"),
    ("taitin", "taitnithe"),
    ("inis", "inste"),
    // 1st conj: -iaigh (strip -igh, broad suffix)
    ("iaigh", "iata"),
    // 1st conj: irregular -gh (non-standard strip or suffix)
    ("comhthogh", "comhthofa"),
    ("díbhlogh", "díbhloghta"),
    ("diongaibh", "diongbháilte"),
    ("guailleáil", "guilleáilte"),
    ("fuirsigh", "fuirste"),
    ("sleabhac", "sleabhctha"),
    ("comhaill", "comhalta"),
    ("creapaill", "creapalta"),
    ("saighid", "saighdte"),
    ("taighid", "taighdte"),
    ("fadhbh", "fadhbhtha"),
    ("scrabh", "scrabhaite"),
    ("riagh", "riaghtha"),
    ("blogh", "bloghta"),
    ("aisig", "aiseagtha"),
    ("clíth", "clite"),
    ("glámh", "glámhtha"),
    ("togh", "tofa"),
    ("bogh", "boghtha"),
    ("logh", "loghtha"),
    ("rígh", "ríthe"),
    // 1st conj: -th/-dh irregular (ite instead of ta, or strip -dh)
    ("eisréidh", "eisréite"),
    ("núth", "núite"),
    ("dath", "daite"),
    // 1st conj: -mhair false match with VA_BROADEN_ROOTS ("mhair"→"mhar")
    ("chomhair", "chomhairthe"),
    ("comhair", "comhairthe"),
    // 1st conj: -mhain verb that broadens for VA (damhain does not)
    ("tamhain", "tamhanta"),
    // 1st conj: verbs where conjugation broadening is wrong for VA
    ("teasairg", "teasairgthe"),
    ("toirmisc", "toirmiscthe"),
    ("dearlaic", "dearlaicthe"),
    ("cumaisc", "cumaiscthe"),
    ("agaill", "agaillte"),
    ("buain", "buainte"),
];

fn va_stem_first(lemma: &str) -> String {
    let l = lemma.to_lowercase();

    for &(slender, broad) in VA_BROADEN_ROOTS {
        if l.ends_with(slender) {
            let prefix = &lemma[..lemma.len() - slender.len()];
            return format!("{}{}", prefix, broad);
        }
    }

    // annáil is the sole -áil verb that broadens for VA (pannáil etc. do not)
    if l == "annáil" {
        return opers::broaden(lemma);
    }

    // Conjugation broadening: correct for VA except monosyllabic -áil/-eáil
    // with a vowel before the suffix (búáil, spraeáil etc. keep slender for VA)
    if needs_broadening_first(lemma) {
        let skip = !opers::polysyllabic(lemma) && ["áil", "eáil"].iter().any(|suf| {
            l.ends_with(suf) && {
                let before = &l[..l.len() - suf.len()];
                before.chars().last().map_or(false, |c| opers::VOWELS.contains(c))
            }
        });
        if !skip {
            return opers::broaden(lemma);
        }
    }

    if needs_syncope_first(lemma)
        && !l.ends_with("mhain")
        && !l.ends_with("eighil")
        && !l.ends_with("uighill")
    {
        return opers::broaden(lemma);
    }

    if opers::polysyllabic(lemma) && !l.ends_with("iomáin") {
        if l.ends_with("óil") || l.ends_with("úil")
            || l.ends_with("áin") || l.ends_with("eáin")
        {
            return opers::broaden(lemma);
        }
    }

    if lookup_deep_syncope(&l).is_some()
        && !l.ends_with("adhain")
        && !l.ends_with("deighil")
        && !l.ends_with("comhair")
    {
        return opers::broaden(lemma);
    }

    lemma.to_string()
}

fn va_suffix(stem: &str) -> &'static str {
    let l = stem.to_lowercase();

    // Consonant cluster -rn follows the r-rule (tha/the), not n-rule (ta/te)
    if l.ends_with("rn") {
        return if opers::is_slender(stem) { "the" } else { "tha" };
    }

    // Digraphs
    if l.ends_with("ch") {
        return if opers::is_slender(stem) { "te" } else { "ta" };
    }
    if l.ends_with("dh") {
        return if opers::is_slender(stem) { "te" } else { "ta" };
    }

    // Last character determines suffix class
    let sl = opers::is_slender(stem);
    match l.chars().last() {
        Some('d' | 'l' | 'n' | 's' | 't') => if sl { "te" } else { "ta" },
        Some('b' | 'c' | 'g' | 'm' | 'p' | 'r') => if sl { "the" } else { "tha" },
        _ => if sl { "te" } else { "ta" },
    }
}

fn va_first(lemma: &str) -> Option<String> {
    let l = lemma.to_lowercase();

    // saigh keeps the -gh in its VA (saighte, not saite)
    if l == "saigh" {
        return Some("saighte".to_string());
    }

    // -igh/-gh endings: strip -gh, add -te
    if l.ends_with("gh") {
        let base = &lemma[..lemma.len() - 2];
        return Some(format!("{}te", base));
    }

    // -bh ending: strip bh, add fa/fe
    if l.ends_with("bh") {
        let base = &lemma[..lemma.len() - 2];
        let sl = if opers::ends_vowel(base) {
            base.chars().last().map_or(false, |c| "eiéí".contains(c))
        } else {
            opers::is_slender(base)
        };
        return Some(format!("{}{}", base, if sl { "fe" } else { "fa" }));
    }

    // -mh ending: strip mh, add fa/fe
    if l.ends_with("mh") {
        let base = &lemma[..lemma.len() - 2];
        let sl = if opers::ends_vowel(base) {
            base.chars().last().map_or(false, |c| "eiéí".contains(c))
        } else {
            opers::is_slender(base)
        };
        return Some(format!("{}{}", base, if sl { "fe" } else { "fa" }));
    }

    // -f ending: strip f, add fa
    if l.ends_with('f') {
        let base = &lemma[..lemma.len() - 1];
        return Some(format!("{}fa", base));
    }

    // -th ending (but not -cht): strip th, add ta/te
    if l.ends_with("th") && !l.ends_with("cht") {
        let base = &lemma[..lemma.len() - 2];
        let sl = if opers::ends_vowel(base) {
            base.chars().last().map_or(false, |c| "eiéí".contains(c))
        } else {
            opers::is_slender(base)
        };
        return Some(format!("{}{}", base, if sl { "te" } else { "ta" }));
    }

    // Compute stem (possibly broadened)
    let stem = va_stem_first(lemma);
    let suffix = va_suffix(&stem);

    // No double-t: stem ending in t + suffix starting with t → drop one
    if l.ends_with('t') && suffix.starts_with('t') {
        Some(format!("{}{}", stem, &suffix[1..]))
    } else {
        Some(format!("{}{}", stem, suffix))
    }
}

fn va_second(lemma: &str) -> Option<String> {
    let l = lemma.to_lowercase();

    // -igh/-aigh: strip -gh, add -the
    if l.ends_with("gh") {
        let base = &lemma[..lemma.len() - 2];
        return Some(format!("{}the", base));
    }

    // -il ending: stem + te (no depalatalisation)
    if l.ends_with("il") {
        return Some(format!("{}te", lemma));
    }

    // -is ending: stem + te
    if l.ends_with("is") {
        return Some(format!("{}te", lemma));
    }

    // -in ending: depalatalise + ta
    if l.ends_with("in") {
        let broad = opers::broaden(lemma);
        return Some(format!("{}ta", broad));
    }

    // -ir ending: depalatalise + tha
    if l.ends_with("ir") {
        let broad = opers::broaden(lemma);
        return Some(format!("{}tha", broad));
    }

    // -im ending: depalatalise + tha
    if l.ends_with("im") {
        let broad = opers::broaden(lemma);
        return Some(format!("{}tha", broad));
    }

    // -ing ending: stem + the
    if l.ends_with("ing") {
        return Some(format!("{}the", lemma));
    }

    // Fallback: use 1st-conj-style suffix rules
    let suffix = va_suffix(lemma);
    Some(format!("{}{}", lemma, suffix))
}

const VA_EXACT: &[(&str, &str)] = &[
    ("iopnóisigh", "hiopnóisithe"),
    ("iodráitigh", "hiodráitithe"),
    ("idriginigh", "hidriginithe"),
    ("eilléanaigh", "Heilléanaithe"),
    ("aigleáil", "haigleáilte"),
    ("aicleáil", "haicleáilte"),
    ("íleáil", "híleáilte"),
    ("inigh", "hinithe"),
    ("apáil", "hapáilte"),
];

fn generate_verbal_adjective(lemma: &str, class: VerbConjugationClass) -> Vec<Form> {
    let l = lemma.to_lowercase();
    for &(word, va_form) in VA_EXACT {
        if l == word {
            return vec![Form::new(va_form)];
        }
    }
    for &(suffix, va_form) in VA_OVERRIDES {
        if l.ends_with(suffix) {
            let prefix = &lemma[..lemma.len() - suffix.len()];
            let result = format!("{}{}", prefix, va_form);
            return vec![Form::new(&result)];
        }
    }

    let va = match class {
        VerbConjugationClass::First => va_first(lemma),
        VerbConjugationClass::Second => va_second(lemma),
        VerbConjugationClass::Irregular => None,
    };
    va.map(|s| vec![Form::new(&s)]).unwrap_or_default()
}

// ---- Verbal noun generation ----

/// Verbal noun overrides: suffix-matched, longest first.
/// Compounds inherit root-verb VN via suffix matching.
const VN_OVERRIDES: &[(&str, &str)] = &[
    // -scríobh compounds: VN = lemma (12 verbs)
    ("scríobh", "scríobh"),
    // -gabh/-ghabh compounds: VN = root + -áil (9+1 verbs)
    ("ghabh", "ghabháil"),
    ("gabh", "gabháil"),
    // -cuir/-chuir compounds: broaden → -cur/-chur (14 verbs)
    ("chuir", "chur"),
    ("cuir", "cur"),
    // -tóg compounds: VN = root + -áil
    ("tóg", "tógáil"),
    // -déan/-dhéan compounds: VN = root + -amh
    ("dhéan", "dhéanamh"),
    ("déan", "déanamh"),
    // -mheas compounds: VN = lemma
    ("mheas", "mheas"),
    // -coimhéad/-choimhéad: VN = lemma
    ("choimhéad", "choimhéad"),
    ("coimhéad", "coimhéad"),
    // -íoc/-ísíoc: VN = lemma (but not slíoc/stríoc)
    ("aisíoc", "aisíoc"),
    ("réamhíoc", "réamhíoc"),
    ("díshioc", "díshioc"),
    ("íoc", "íoc"),
    // -díol/-dhíol: VN = lemma
    ("dhíol", "dhíol"),
    ("díol", "díol"),
    // -roinn: VN = -roinnt (but not sloinn)
    ("sloinn", "sloinneadh"),
    ("roinn", "roinnt"),
    // -leag: VN = -leagan
    ("leag", "leagan"),
    // -teilg: VN = -teilgean
    ("teilg", "teilgean"),
    // -suigh/-shuigh: VN = strip -igh → -í
    ("shuigh", "shuí"),
    ("suigh", "suí"),
    // -thit/-tit: VN = -titim
    ("thit", "thitim"),
    ("tit", "titim"),
    // -scoir: VN = -scor
    ("scoir", "scor"),
    // -iompair: VN = -iompar
    ("iompair", "iompar"),
    // -tionóil: VN = -tionól
    ("tionóil", "tionól"),
    // -imir: VN = -imirt
    ("imir", "imirt"),
    // -ghair: VN = -ghairm (but not faghair/ionghair/urghair)
    ("ionghair", "ionghaire"),
    ("urghair", "urghaire"),
    ("faghair", "faghairt"),
    ("ghair", "ghairm"),
    // -siúil: VN = -siúl
    ("siúil", "siúl"),
    // -ceangail/-cheangail: VN = -ceangal/-cheangal
    ("cheangail", "cheangal"),
    ("ceangail", "ceangal"),
    // -soláthair: VN = -soláthar
    ("soláthair", "soláthar"),
    // -buail: VN = -bualadh
    ("buail", "bualadh"),
    // -fair: VN = -faire
    ("cúlfhair", "cúlfhaire"),
    ("fair", "faire"),
    // -dáil (the verb dáil, not -dáil loanwords which are handled as -áil VN=lemma)
    ("athdháil", "athdháileadh"),
    // -buail/-bhuail: VN = -bualadh
    ("bhuail", "bhualadh"),
    ("buail", "bualadh"),
    // VN=lemma compound families (safe suffix length)
    ("bhruith", "bhruith"),
    ("bruith", "bruith"),
    ("ghoin", "ghoin"),
    ("mheas", "mheas"),
    ("choimhéad", "choimhéad"),
    ("coimhéad", "coimhéad"),
    ("oimeád", "oimeád"),
    ("teagasc", "teagasc"),
    ("aithris", "aithris"),
    ("isnéis", "isnéis"),
    ("úsáid", "úsáid"),
    ("triail", "triail"),
    ("riar", "riar"),
    // Miscellaneous consistent families
    ("druid", "druidim"),
    ("crith", "crith"),
    ("diall", "diall"),
    ("triall", "triall"),
    // -sheinn: VN = -sheinm
    ("sheinn", "sheinm"),
    ("seinn", "seinm"),
    // -chuntais: VN = -chuntas
    ("chuntais", "chuntas"),
    ("cuntais", "cuntas"),
    // 2nd conj -suigh/-shuigh/-luigh: strip -igh, add -í
    ("shuigh", "shuí"),
    ("suigh", "suí"),
    ("luigh", "luí"),
    // 2nd conj -éirigh: strip -igh, add -í
    ("éirigh", "éirí"),
    // 2nd conj -ionsaigh: strip -aigh, add -aí
    ("ionsaigh", "ionsaí"),
    // 2nd conj -igh → -í (monosyllabic & other specific verbs)
    ("achainigh", "achainí"),
    ("ceasnaigh", "ceasnaí"),
    ("corraigh", "corraí"),
    ("cónaigh", "cónaí"),
    ("dligh", "dlí"),
    ("dluigh", "dluí"),
    ("eascainigh", "eascainí"),
    ("fiafraigh", "fiafraí"),
    ("fionraigh", "fionraí"),
    ("guigh", "guí"),
    ("impigh", "impí"),
    ("snoigh", "snoí"),
    ("taithigh", "taithí"),
    // 2nd conj -ceannaigh: strip -aigh, add -ach
    ("cheannaigh", "cheannach"),
    ("ceannaigh", "ceannach"),
    // 2nd conj -clúdaigh: strip -aigh, add -ach
    ("chlúdaigh", "chlúdach"),
    ("clúdaigh", "clúdach"),
    // 2nd conj -réitigh: strip -igh, add -each
    ("réitigh", "réiteach"),
    // 2nd conj -igh → -ach (individual verbs)
    ("amhastraigh", "amhastrach"),
    ("aslaigh", "aslach"),
    ("baslaigh", "baslach"),
    ("báistigh", "báisteach"),
    ("crústaigh", "crústach"),
    ("cuardaigh", "cuardach"),
    ("cumhdaigh", "cumhdach"),
    ("díoscarnaigh", "díoscarnach"),
    ("eitigh", "eiteach"),
    ("fuadaigh", "fuadach"),
    ("fuirigh", "fuireach"),
    ("taifigh", "taifeach"),
    ("taithmhigh", "taithmheach"),
    ("toibhigh", "tobhach"),
    ("éagnaigh", "éagnach"),
    // 2nd conj -léigh → -léamh (compound family)
    ("léigh", "léamh"),
    // 2nd conj -smaoinigh → -smaoineamh
    ("smaoinigh", "smaoineamh"),
    // 2nd conj -machnaigh → -machnamh
    ("mhachnaigh", "mhachnamh"),
    ("machnaigh", "machnamh"),
    // 2nd conj -igh → -eamh/-amh (individual verbs)
    ("caidrigh", "caidreamh"),
    ("cuimhnigh", "cuimhneamh"),
    ("cúisigh", "cúiseamh"),
    ("cúitigh", "cúiteamh"),
    ("cúnaigh", "cúnamh"),
    ("dealraigh", "dealramh"),
    ("foighnigh", "foighneamh"),
    ("fuaidrigh", "fuaidreamh"),
    ("fritháirigh", "fritháireamh"),
    ("foréiligh", "foréileamh"),
    ("frithéiligh", "frithéileamh"),
    ("míshásaigh", "míshásamh"),
    ("sásaigh", "sásamh"),
    ("taibhrigh", "taibhreamh"),
    ("táinsigh", "táinseamh"),
    ("téarnaigh", "téarnamh"),
    ("tórraigh", "tórramh"),
    ("áirigh", "áireamh"),
    // 2nd conj -igh → -chan (vowel + -igh → strip, long vowel + -chan)
    ("beoigh", "beochan"),
    ("cruaigh", "cruachan"),
    ("dubhaigh", "dúchan"),
    ("buaigh", "buachan"),
    ("ruaigh", "ruachan"),
    ("láigh", "láchan"),
    ("tiubhaigh", "tiúchan"),
    // 2nd conj misc -igh overrides
    ("admhaigh", "admháil"),
    ("airigh", "aireachtáil"),
    ("coinnigh", "coinneáil"),
    ("sholáthraigh", "sholáthar"),
    ("soláthraigh", "soláthar"),
    // 2nd conj -igh → -eacht
    ("dhúisigh", "dhúiseacht"),
    ("dúisigh", "dúiseacht"),
    ("aoirigh", "aoireacht"),
    ("mháistrigh", "mháistreacht"),
    ("máistrigh", "máistreacht"),
    // 2nd conj -igh → -iúint
    ("eisigh", "eisiúint"),
    // --- Compound families: -scaoil → -scaoileadh (NOT -scaoilt) ---
    ("scaoil", "scaoileadh"),
    // --- Compound families: -fuaigh → -fuáil ---
    ("fhuaigh", "fhuáil"),
    ("fuaigh", "fuáil"),
    // --- Compound families: -teilg → -teilgean ---
    // already have ("teilg", "teilgean") above
    // --- Compound families: -lig/-eislig → -ligean ---
    ("eislig", "eisligean"),
    ("folig", "foligean"),
    ("tarmlig", "tarmligean"),
    ("lig", "ligean"),
    // --- 1st conj -isc/-áisc → broaden: drop slender, -scadh ---
    // These verbs slenderize the -sc cluster; VN broadens it back
    ("loisc", "loscadh"),
    ("fháisc", "fháscadh"),
    ("fáisc", "fáscadh"),
    ("rúisc", "rúscadh"),
    ("brúisc", "brúscadh"),
    ("coisc", "cosc"),
    ("cónaisc", "cónascadh"),
    // --- 1st conj -aic/-aisc → broadened (comhrac, cumasc, etc.) ---
    ("comhraic", "comhrac"),
    ("cumaisc", "cumasc"),
    ("iomlaisc", "iomlasc"),
    ("toirmisc", "toirmeasc"),
    ("urchoisc", "urchosc"),
    ("tochais", "tochas"),
    ("tochrais", "tochras"),
    ("díthochais", "díthochas"),
    ("díthochrais", "díthochras"),
    ("tomhais", "tomhas"),
    ("fómhais", "fómhas"),
    ("urmhais", "urmhaise"),
    ("súraic", "súrac"),
    // --- 1st conj -loit/-aghloit → broadened -lot/-aghlot ---
    ("loit", "lot"),
    // --- -glaoigh → -glaoch ---
    ("ghlaoigh", "ghlaoch"),
    ("glaoigh", "glaoch"),
    // --- -pléigh → -phlé (strip -igh, long vowel root) ---
    ("phléigh", "phlé"),
    ("pléigh", "plé"),
    // --- -beoigh/-reoigh/-dreoigh/-feoigh/-sceoigh/-glaeigh/-breoigh → strip -igh ---
    ("beoigh", "beo"),
    ("bheoigh", "bheochan"),
    ("reoigh", "reo"),
    ("dreoigh", "dreo"),
    ("feoigh", "feo"),
    ("sceoigh", "sceo"),
    ("glaeigh", "glae"),
    ("breoigh", "breo"),
    ("díreoigh", "díreo"),
    ("athreoigh", "athreo"),
    // --- -igh → -ígheadh (some long-root first conj) ---
    ("rígh", "rí"),
    ("athrígh", "athrí"),
    ("cloígh", "cloí"),
    ("cnaígh", "cnaí"),
    ("maígh", "maíomh"),
    // --- 1st conj -ith/-eith/-aith → broadened -amh/adh forms ---
    ("chaith", "chaitheamh"),
    ("caith", "caitheamh"),
    ("maith", "maitheamh"),
    ("braith", "brath"),
    ("scraith", "scrathadh"),
    ("snáith", "snáthadh"),
    ("feith", "feitheamh"),
    // --- 1st conj -idh → broadened -dh forms ---
    ("luaidh", "luadh"),
    ("iomráidh", "iomrádh"),
    ("claidh", "claidhe"),
    ("slaidh", "slaidhe"),
    ("eisréidh", "eisréadh"),
    ("leoidh", "leodh"),
    // --- 1st conj -ibh → -ibhe ---
    ("ibh", "ibhe"),
    ("díbh", "díbhe"),
    ("diongaibh", "diongbháil"),
    // --- 2nd conj -airigh → -airiú (not -aireamh) ---
    ("allmhairigh", "allmhairiú"),
    ("athallmhairigh", "athallmhairiú"),
    ("athonnmhairigh", "athonnmhairiú"),
    ("onnmhairigh", "onnmhairiú"),
    ("ríomhairigh", "ríomhairiú"),
    ("guairigh", "guairiú"),
    ("lúcháirigh", "lúcháiriú"),
    ("náirigh", "náiriú"),
    ("adhnáirigh", "adhnáiriú"),
    ("eiseamláirigh", "eiseamláiriú"),
    // 2nd conj -léirigh → -léiriú
    ("léirigh", "léiriú"),
    // 2nd conj specific -igh → -iú (not default -iúint or -í)
    ("athdheisigh", "athdheisiú"),
    ("deisigh", "deisiú"),
    ("breisigh", "breisiú"),
    ("treisigh", "treisiú"),
    ("cleitigh", "cleitiú"),
    ("coincréitigh", "coincréitiú"),
    ("díréitigh", "díréitiú"),
    ("geoidligh", "geoidliú"),
    ("polaiméirigh", "polaiméiriú"),
    ("sionsaigh", "sionsú"),
    // 2nd conj -aigh → -ú (not -ach)
    ("aiceannaigh", "aiceannú"),
    ("ailtéarnaigh", "ailtéarnú"),
    ("bréagnaigh", "bréagnú"),
    ("céaslaigh", "céaslú"),
    ("diamhaslaigh", "diamhaslú"),
    ("maslaigh", "maslú"),
    ("tréaslaigh", "tréaslú"),
    ("reoánaigh", "reoánanú"),
    // 2nd conj -aigh → -aíocht/-aí (activity nouns)
    ("marcaigh", "marcaíocht"),
    ("rothaigh", "rothaíocht"),
    ("rámhaigh", "rámhaíocht"),
    ("tóraigh", "tóraíocht"),
    ("coisigh", "coisíocht"),
    ("osnaigh", "osnaíl"),
    // 2nd conj -aigh → -acht/-t special
    ("damhsaigh", "damhsa"),
    ("cniogdhamhsaigh", "cniogdhamhsa"),
    ("fortaigh", "fortacht"),
    ("tathantaigh", "tathant"),
    ("teastaigh", "teastáil"),
    ("teagmhaigh", "teagmháil"),
    ("giollaigh", "giollacht"),
    // 2nd conj -igh → -eacht special
    ("imigh", "imeacht"),
    ("mainnigh", "mainneachtain"),
    ("tairngir", "tairngireacht"),
    ("gibir", "gibreacht"),
    // 2nd conj: -aigh → special -ughadh (miadhaigh)
    ("miadhaigh", "miadhughadh"),
    // 2nd conj -ir → broadened (not -irt)
    ("bladair", "bladar"),
    ("cogair", "cogar"),
    ("tacair", "tacar"),
    ("togair", "togradh"),
    ("athrómhair", "athrómhar"),
    ("rómhair", "rómhar"),
    ("iomair", "iomramh"),
    // 2nd conj -ir → -radh (syncope)
    ("bladhair", "bladhradh"),
    ("bodhair", "bodhradh"),
    ("cabhair", "cabhradh"),
    ("cnámhair", "cnáimhreadh"),
    ("gleadhair", "gleadhradh"),
    ("leadair", "leadradh"),
    ("lochair", "lochradh"),
    ("sciomair", "sciomradh"),
    ("siabhair", "siabhradh"),
    ("tionnabhair", "tionnabhradh"),
    ("deachair", "deachrú"),
    ("locair", "locrú"),
    ("torchair", "torchra"),
    // 2nd conj -il → broadened (not -ilt)
    ("codail", "codladh"),
    ("cadhail", "caidhleadh"),
    ("freastail", "freastal"),
    ("gogail", "gogal"),
    ("maoscail", "maoscal"),
    ("scobail", "scobladh"),
    ("sárthadhaill", "sárthadhall"),
    ("tadhaill", "tadhall"),
    ("taistil", "taisteal"),
    ("tochsail", "tochsal"),
    ("comhaill", "comhall"),
    ("fuighill", "fuigheall"),
    ("sroighill", "sroighleadh"),
    ("insamhail", "insamhladh"),
    ("déadail", "déadladh"),
    // 2nd conj -in → special (not -int)
    ("ascain", "ascnamh"),
    ("foscain", "foscnamh"),
    ("tionscain", "tionscnamh"),
    ("taitin", "taitneamh"),
    ("stoithin", "stoithneadh"),
    ("tafainn", "tafann"),
    ("ionnail", "ionladh"),
    ("innill", "inleadh"),
    // 2nd conj -is → special (not VN=lemma)
    ("athinis", "athinsint"),
    ("réamhinis", "réamhinsint"),
    ("inis", "insint"),
    // 2nd conj -ir special
    ("athfhaghair", "athfhaghairt"),
    ("coimpir", "coimpeart"),
    // 2nd conj: -iúint verbs
    ("glinnigh", "glinniúint"),
    // 2nd conj: -eamh verbs (2nd conj -cúisigh → -cúiseamh)
    ("díotchúisigh", "díotchúiseamh"),
    ("ionchúisigh", "ionchúiseamh"),
    // 2nd conj: fuirsigh, special -igh → -eadh
    ("fuirsigh", "fuirseadh"),
    // 2nd conj: maistrigh → maistreadh
    ("maistrigh", "maistreadh"),
    // 2nd conj: fordhubhaigh → fordhúchan
    ("fordhubhaigh", "fordhúchan"),
    ("eispéirigh", "eispéiriú"),
    ("doiléirigh", "doiléiriú"),
    ("athléirigh", "athléiriú"),
    ("forléirigh", "forléiriú"),
    ("soiléirigh", "soiléiriú"),
    // 2nd conj: tacmhaing → tacmhang
    ("tacmhaing", "tacmhang"),
];

fn vn_first(lemma: &str) -> Option<String> {
    let l = lemma.to_lowercase();

    // -áil/-eáil verbs: VN = lemma
    if l.ends_with("áil") || l.ends_with("eáil") {
        return Some(lemma.to_string());
    }

    // -igh verbs (monosyllabic in 1st conj)
    if l.ends_with("igh") {
        let root = &lemma[..lemma.len() - 3];
        let root_l = &l[..l.len() - 3];
        // Long vowel root: strip -igh (dóigh→dó, luaigh→lua)
        if root_l.ends_with(|c: char| "áéíóú".contains(c))
            || root_l.ends_with("ua")
            || root_l.ends_with("ao")
        {
            return Some(root.to_string());
        }
        // Short vowel root: strip -igh, add -í (suigh→suí, nigh→ní)
        return Some(format!("{}í", root));
    }

    // 1st conj verbs ending in -il/-in/-ir/-ain/-áin/-óin/-ing/-im:
    // VN = lemma + -t (same pattern as 2nd conj -il/-in/-ir)
    // adhain→adhaint, argóin→argóint, labhair→labhairt, oscail→oscailt
    if l.ends_with("il") || l.ends_with("in") || l.ends_with("ir")
        || l.ends_with("im") || l.ends_with("ing")
    {
        return Some(format!("{}t", lemma));
    }

    // Default: broaden + -adh (broad) or stem + -eadh (slender)
    let stem = if needs_broadening_first(lemma) {
        opers::broaden(lemma)
    } else if needs_syncope_first(lemma) {
        opers::broaden(lemma)
    } else {
        lemma.to_string()
    };

    if opers::is_slender(&stem) {
        Some(format!("{}eadh", stem))
    } else {
        Some(format!("{}adh", stem))
    }
}

fn vn_second(lemma: &str) -> Option<String> {
    let l = lemma.to_lowercase();

    // -áil/-eáil verbs: VN = lemma
    if l.ends_with("áil") || l.ends_with("eáil") {
        return Some(lemma.to_string());
    }

    // -aigh: strip -aigh, add -ú (achtaigh → achtú)
    // -igh (non-aigh): strip -igh, add -iú (Laidinigh → Laidiniú)
    if l.ends_with("aigh") {
        let base = &lemma[..lemma.len() - 4];
        return Some(format!("{}ú", base));
    }
    if l.ends_with("igh") {
        let base = &lemma[..lemma.len() - 3];
        return Some(format!("{}iú", base));
    }

    // -il/-in/-ir/-im: lemma + -t (oscail→oscailt, imir→imirt, agair→agairt)
    if l.ends_with("il") || l.ends_with("in") || l.ends_with("ir") || l.ends_with("im") {
        return Some(format!("{}t", lemma));
    }

    // -is: VN = lemma (aithris→aithris)
    if l.ends_with("is") {
        return Some(lemma.to_string());
    }

    // -ing: lemma + -t
    if l.ends_with("ing") {
        return Some(format!("{}t", lemma));
    }

    // Fallback
    if opers::is_slender(lemma) {
        Some(format!("{}eadh", lemma))
    } else {
        Some(format!("{}adh", lemma))
    }
}

const VN_EXACT: &[(&str, &str)] = &[
    // Standalone verbs with VN=lemma
    ("achomharc", "achomharc"), ("agóid", "agóid"), ("amharc", "amharc"),
    ("aisíoc", "aisíoc"), ("aisléim", "aisléim"), ("aisling", "aisling"),
    ("aitheasc", "aitheasc"), ("barúil", "barúil"), ("broic", "broic"),
    ("buain", "buain"), ("bruíon", "bruíon"), ("cac", "cac"),
    ("casaoid", "casaoid"), ("clíth", "clíth"), ("conspóid", "conspóid"),
    ("dearmad", "dearmad"), ("díol", "díol"), ("díon", "díon"),
    ("díoghail", "díoghail"), ("díolaim", "díolaim"), ("díospóid", "díospóid"),
    ("díshioc", "díshioc"), ("diúl", "diúl"), ("dord", "dord"),
    ("dréim", "dréim"), ("éag", "éag"), ("faichill", "faichill"),
    ("faisnéis", "faisnéis"), ("fás", "fás"), ("feighil", "feighil"),
    ("fiach", "fiach"), ("foghlaim", "foghlaim"), ("gad", "gad"),
    ("gearán", "gearán"), ("goid", "goid"), ("idircheart", "idircheart"),
    ("inghreim", "inghreim"), ("lámhach", "lámhach"), ("leigheas", "leigheas"),
    ("léim", "léim"), ("léirscrios", "léirscrios"), ("líomhain", "líomhain"),
    ("lorg", "lorg"), ("meath", "meath"), ("mún", "mún"),
    ("ól", "ól"), ("pocléim", "pocléim"), ("reic", "reic"),
    ("réamhíoc", "réamhíoc"), ("rith", "rith"), ("ríomh", "ríomh"),
    ("seilg", "seilg"), ("sioc", "sioc"), ("slad", "slad"),
    ("snámh", "snámh"), ("sníomh", "sníomh"), ("stad", "stad"),
    ("tathaoir", "tathaoir"), ("teip", "teip"), ("toghail", "toghail"),
    ("tomhaidhm", "tomhaidhm"), ("tonach", "tonach"),
    ("tost", "tost"), ("trácht", "trácht"), ("tréthál", "tréthál"),
    ("triosc", "triosc"), ("troid", "troid"), ("trust", "trust"),
    ("tuar", "tuar"), ("tál", "tál"), ("tóch", "tóch"),
    ("urbhac", "urbhac"), ("íoc", "íoc"),
    // Standalone verb overrides
    ("dáil", "dáileadh"),
    ("annáil", "annáladh"),
    ("figh", "fí"), ("ligh", "lí"), ("nigh", "ní"), ("snigh", "sní"),
    ("faigh", "fáil"),
    ("righ", "ríochan"),
    ("éiligh", "éileamh"),
    ("iaigh", "iamh"), ("eisiaigh", "eisiamh"),
    ("foriaigh", "foriamh"), ("iniaigh", "iniamh"),
    // 1st conj -ir/-il that DON'T take -t (irregular stem changes)
    ("air", "ar"),
    ("athghoin", "athghoin"),
    // 1st conj irregular VN singletons
    ("adhair", "adhradh"),
    ("agaill", "agallamh"),
    ("aghloit", "aghlot"),
    ("ainic", "anacal"),
    ("aisig", "aiseag"),
    ("arg", "argain"),
    ("bligh", "bleán"),
    ("broim", "bromadh"),
    ("buair", "buaireamh"),
    ("buígh", "buíochan"),
    ("búir", "búireadh"),
    ("caoin", "caoineadh"),
    ("car", "carthain"),
    ("ceiliúir", "ceiliúradh"),
    ("ceis", "ceasacht"),
    ("cin", "cineadh"),
    ("cinnir", "cinnireacht"),
    ("cling", "clingeadh"),
    ("cnead", "cneadach"),
    ("coir", "cor"),
    ("coisric", "coisreacan"),
    ("comhair", "comhaireamh"),
    ("athchomhair", "athchomhaireamh"),
    ("comóir", "comóradh"),
    ("comhthiúin", "comhthiúnadh"),
    ("creid", "creidiúint"),
    ("díchreid", "díchreidiúint"),
    ("creim", "creimeadh"),
    ("cráin", "cráineadh"),
    ("cáin", "cáineadh"),
    ("damhain", "damhnadh"),
    ("deil", "deileadh"),
    ("deimhneasc", "deimhneasc"),
    ("ding", "dingeadh"),
    ("doir", "dor"),
    ("dámh", "dámhachtain"),
    ("eisiacht", "eisiachtain"),
    ("eisil", "eisileadh"),
    ("fan", "fanacht"),
    ("feil", "feiliúint"),
    ("feir", "feireadh"),
    ("fionnachtain", "fionnachtaineadh"),
    ("fodháil", "fodháileadh"),
    ("imdháil", "imdháileadh"),
    ("leithdháil", "leithdháileadh"),
    ("folean", "foleanúint"),
    ("fordhing", "fordhingeadh"),
    ("forthairg", "forthairiscint"),
    ("fuill", "fuilleamh"),
    ("fuin", "fuineadh"),
    ("fág", "fágáil"),
    ("fáir", "fáireadh"),
    ("féach", "féachaint"),
    ("féad", "féadachtáil"),
    ("fóin", "fónamh"),
    ("fóir", "fóirithint"),
    ("gair", "gairm"),
    ("gin", "giniúint"),
    ("athghin", "athghiniúint"),
    ("glam", "glamaíl"),
    ("glean", "gleanúint"),
    ("nasclean", "nascleanúint"),
    ("gluais", "gluaiseacht"),
    ("toghluais", "toghluasacht"),
    ("goil", "gol"),
    ("goin", "goin"),
    ("gáir", "gáire"),
    ("géim", "géimneach"),
    ("iarr", "iarraidh"),
    ("iasc", "iascach"),
    ("imdheaghail", "imdheaghail"),
    ("imdhruid", "imdhruidim"),
    ("imthnúth", "imthnúth"),
    ("insil", "insileadh"),
    ("lean", "leanúint"),
    ("ling", "lingeadh"),
    ("mair", "maireachtáil"),
    ("meas", "meas"),
    ("míriar", "míriaradh"),
    ("múin", "múineadh"),
    ("oil", "oiliúint"),
    ("athoil", "athoiliúint"),
    ("oir", "oiriúint"),
    ("oiris", "oiriseamh"),
    ("tairis", "tairiseamh"),
    ("pláigh", "plá"),
    ("truaigh", "trua"),
    ("rinc", "rince"),
    ("saigh", "saighe"),
    ("saighid", "saighdeadh"),
    ("sceamh", "sceamhaíl"),
    ("scil", "scileadh"),
    ("scread", "screadach"),
    ("scréach", "scréachach"),
    ("scáin", "scáineadh"),
    ("seas", "seasamh"),
    ("sil", "sileadh"),
    ("sir", "sireadh"),
    ("diansir", "diansireadh"),
    ("sligh", "slighe"),
    ("slíoc", "slíocadh"),
    ("stríoc", "stríocadh"),
    ("spadhar", "spadhradh"),
    ("speir", "speireadh"),
    ("stiúir", "stiúradh"),
    ("substain", "substaineadh"),
    ("síobshiúil", "síobshiúl"),
    ("taighid", "taighde"),
    ("tairg", "tairiscint"),
    ("garbhtheilg", "garbhtheilgean"),
    ("dísletheilg", "dísletheilgean"),
    ("imthairg", "imthairiscint"),
    ("réamhtheilg", "réamhtheilgean"),
    ("rótheilg", "rótheilgean"),
    ("tíoptheilg", "tíoptheilgean"),
    ("tamhain", "tamhnadh"),
    ("teasairg", "teasargan"),
    ("til", "tileadh"),
    ("tionlaic", "tionlacan"),
    ("tiúin", "tiúnadh"),
    ("tnúth", "tnúth"),
    ("tonnchrith", "tonnchrith"),
    ("treapáin", "treapánadh"),
    ("tréig", "tréigean"),
    ("tubh", "tubha"),
    ("tuig", "tuiscint"),
    ("tuil", "tuile"),
    ("tuill", "tuilleamh"),
    ("táir", "táireadh"),
    ("uaim", "uamadh"),
    ("urlaic", "urlacan"),
    ("sleabhac", "sleabhcadh"),
    ("éagaoin", "éagaoineadh"),
    ("éist", "éisteacht"),
    ("athéist", "athéisteacht"),
    ("cúléist", "cúléisteacht"),
    ("úim", "úmadh"),
    // h-prothesis VN (BuNaMo stores h-prefixed form)
    ("aicleáil", "haicleáil"),
    ("aigleáil", "haigleáil"),
    ("apáil", "hapáil"),
    ("íleáil", "híleáil"),
    // 1st conj -igh compounds classified as 1st conj
    ("athbheoigh", "athbheochan"),
    ("athnuaigh", "athnuachan"),
    ("creapaill", "creapall"),
    ("díchoisric", "díchoisreacan"),
    // -éigh → -éamh (not strip-igh; these want root+amh)
    ("éigh", "éamh"),
    ("atéigh", "atéamh"),
    ("forthéigh", "forthéamh"),
    ("róthéigh", "róthéamh"),
    // Specific -igh singletons
    ("rígh", "rí"),
    ("athrígh", "athrí"),
    ("cloígh", "cloí"),
    ("cnaígh", "cnaí"),
    ("maígh", "maíomh"),
    ("buígh", "buíochan"),
    ("pláigh", "plá"),
    ("truaigh", "trua"),
    // Remaining singletons
    ("tibh", "tibheadh"),
    ("scillig", "scilligeadh"),
    ("troisc", "troscadh"),
    // 2nd conj singletons not covered by suffix patterns
    ("athstóraigh", "athstórú"),
    ("gluaisrothaigh", "gluaisrothú"),
    ("caithréimigh", "caithréimiú"),
    ("caochfháithimigh", "caochfháithimiú"),
    ("cimigh", "cimiú"),
    ("comhshuimigh", "comhshuimiú"),
    ("ionstraimigh", "ionstraimiú"),
    ("suimigh", "suimiú"),
    ("tuairimigh", "tuairimiú"),
    ("éimigh", "éimiú"),
    ("imaistrigh", "imaistriú"),
    ("eilléanaigh", "Heilléanú"),
    // h-prothesis 2nd conj VN
    ("idriginigh", "hidriginiú"),
    ("inigh", "hiniú"),
    ("iodráitigh", "hiodráitiú"),
    ("iopnóisigh", "hiopnóisiú"),
];

fn generate_verbal_noun(lemma: &str, class: VerbConjugationClass) -> Vec<Form> {
    let l = lemma.to_lowercase();
    for &(word, vn_form) in VN_EXACT {
        if l == word {
            return vec![Form::new(vn_form)];
        }
    }
    // Find longest matching suffix (most specific wins)
    let mut best: Option<(&str, &str)> = None;
    for &(suffix, vn_form) in VN_OVERRIDES {
        if l.ends_with(suffix) {
            if best.map_or(true, |(s, _)| suffix.len() > s.len()) {
                best = Some((suffix, vn_form));
            }
        }
    }
    if let Some((suffix, vn_form)) = best {
        let prefix = &lemma[..lemma.len() - suffix.len()];
        let result = format!("{}{}", prefix, vn_form);
        return vec![Form::new(&result)];
    }

    let vn = match class {
        VerbConjugationClass::First => vn_first(lemma),
        VerbConjugationClass::Second => vn_second(lemma),
        VerbConjugationClass::Irregular => None,
    };
    vn.map(|s| vec![Form::new(&s)]).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irregulars() {
        assert_eq!(guess_conjugation("bí"), VerbConjugationClass::Irregular);
        assert_eq!(guess_conjugation("abair"), VerbConjugationClass::Irregular);
        assert_eq!(guess_conjugation("déan"), VerbConjugationClass::Irregular);
    }

    #[test]
    fn test_monosyllabic_first() {
        // Single vowel group → 1st
        assert_eq!(guess_conjugation("mol"), VerbConjugationClass::First);
        assert_eq!(guess_conjugation("bris"), VerbConjugationClass::First);
        assert_eq!(guess_conjugation("cuir"), VerbConjugationClass::First);
        assert_eq!(guess_conjugation("léigh"), VerbConjugationClass::First);
    }

    #[test]
    fn test_igh_second() {
        // Polysyllabic -igh → 2nd
        assert_eq!(guess_conjugation("ceannaigh"), VerbConjugationClass::Second);
        assert_eq!(guess_conjugation("bailigh"), VerbConjugationClass::Second);
    }

    #[test]
    fn test_syncopating_second() {
        // -il/-in/-ir/-is → 2nd
        assert_eq!(guess_conjugation("imir"), VerbConjugationClass::Second);
        assert_eq!(guess_conjugation("oscail"), VerbConjugationClass::Second);
        assert_eq!(guess_conjugation("cosain"), VerbConjugationClass::Second);
    }

    #[test]
    fn test_long_vowel_first() {
        // -áil → 1st (not syncopating)
        assert_eq!(guess_conjugation("sábháil"), VerbConjugationClass::First);
    }

    #[test]
    fn test_default_first() {
        assert_eq!(guess_conjugation("tosaigh"), VerbConjugationClass::Second);
    }

    #[test]
    fn test_future_method() {
        // mol → molfaidh (has 'f' → 1st)
        assert_eq!(
            get_conjugation_from_future("mol", "molfaidh"),
            VerbConjugationClass::First
        );
        // ceannaigh → ceannóidh (no 'f' → 2nd)
        assert_eq!(
            get_conjugation_from_future("ceannaigh", "ceannóidh"),
            VerbConjugationClass::Second
        );
    }

    #[test]
    fn test_vowel_groups() {
        assert_eq!(count_vowel_groups("mol"), 1);
        assert_eq!(count_vowel_groups("oscail"), 2);
        assert_eq!(count_vowel_groups("ceannaigh"), 2);
        assert_eq!(count_vowel_groups("bí"), 1);
    }

    // -----------------------------------------------------------------
    // Verb paradigm tests — validated against BuNaMo XML
    // -----------------------------------------------------------------

    fn f(v: &[Form]) -> &str {
        &v[0].value
    }

    // --- 1st conjugation, broad stem: mol ---

    #[test]
    fn test_mol_past() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.base), "mol");
        assert_eq!(f(&v.past.indep.pl1),  "molamar");
        assert_eq!(f(&v.past.indep.pl3),  "moladar");
        assert_eq!(f(&v.past.indep.auto), "moladh");
    }

    #[test]
    fn test_mol_past_cont() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.base), "moladh");
        assert_eq!(f(&v.past_cont.indep.sg1),  "molainn");
        assert_eq!(f(&v.past_cont.indep.sg2),  "moltá");
        assert_eq!(f(&v.past_cont.indep.pl1),  "molaimis");
        assert_eq!(f(&v.past_cont.indep.pl3),  "molaidís");
        assert_eq!(f(&v.past_cont.indep.auto), "moltaí");
    }

    #[test]
    fn test_mol_pres_cont() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.pres_cont.indep.base), "molann");
        assert_eq!(f(&v.pres_cont.indep.sg1),  "molaim");
        assert_eq!(f(&v.pres_cont.indep.pl1),  "molaimid");
        assert_eq!(f(&v.pres_cont.indep.auto), "moltar");
    }

    #[test]
    fn test_mol_fut() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.fut.indep.base), "molfaidh");
        assert_eq!(f(&v.fut.indep.pl1),  "molfaimid");
        assert_eq!(f(&v.fut.indep.auto), "molfar");
    }

    #[test]
    fn test_mol_cond() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.cond.indep.base), "molfadh");
        assert_eq!(f(&v.cond.indep.sg1),  "molfainn");
        assert_eq!(f(&v.cond.indep.sg2),  "molfá");
        assert_eq!(f(&v.cond.indep.pl1),  "molfaimis");
        assert_eq!(f(&v.cond.indep.pl3),  "molfaidís");
        assert_eq!(f(&v.cond.indep.auto), "molfaí");
    }

    #[test]
    fn test_mol_imper() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.imper.base), "moladh");
        assert_eq!(f(&v.imper.sg1),  "molaim");
        assert_eq!(f(&v.imper.sg2),  "mol");
        assert_eq!(f(&v.imper.pl1),  "molaimis");
        assert_eq!(f(&v.imper.pl2),  "molaigí");
        assert_eq!(f(&v.imper.pl3),  "molaidís");
        assert_eq!(f(&v.imper.auto), "moltar");
    }

    #[test]
    fn test_mol_subj() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.subj.base), "mola");
        assert_eq!(f(&v.subj.pl1),  "molaimid");
        assert_eq!(f(&v.subj.auto), "moltar");
    }

    // --- 1st conjugation, slender stem: bris ---

    #[test]
    fn test_bris_past() {
        let v = Verb::from_lemma("bris", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.base), "bris");
        assert_eq!(f(&v.past.indep.pl1),  "briseamar");
        assert_eq!(f(&v.past.indep.pl3),  "briseadar");
        assert_eq!(f(&v.past.indep.auto), "briseadh");
    }

    #[test]
    fn test_bris_pres_cont() {
        let v = Verb::from_lemma("bris", VerbConjugationClass::First);
        assert_eq!(f(&v.pres_cont.indep.base), "briseann");
        assert_eq!(f(&v.pres_cont.indep.sg1),  "brisim");
        assert_eq!(f(&v.pres_cont.indep.pl1),  "brisimid");
        assert_eq!(f(&v.pres_cont.indep.auto), "bristear");
    }

    #[test]
    fn test_bris_fut() {
        let v = Verb::from_lemma("bris", VerbConjugationClass::First);
        assert_eq!(f(&v.fut.indep.base), "brisfidh");
        assert_eq!(f(&v.fut.indep.pl1),  "brisfimid");
        assert_eq!(f(&v.fut.indep.auto), "brisfear");
    }

    #[test]
    fn test_bris_cond() {
        let v = Verb::from_lemma("bris", VerbConjugationClass::First);
        assert_eq!(f(&v.cond.indep.base), "brisfeadh");
        assert_eq!(f(&v.cond.indep.sg1),  "brisfinn");
        assert_eq!(f(&v.cond.indep.sg2),  "brisfeá");
        assert_eq!(f(&v.cond.indep.pl1),  "brisfimis");
        assert_eq!(f(&v.cond.indep.pl3),  "brisfidís");
        assert_eq!(f(&v.cond.indep.auto), "brisfí");
    }

    #[test]
    fn test_bris_imper() {
        let v = Verb::from_lemma("bris", VerbConjugationClass::First);
        assert_eq!(f(&v.imper.base), "briseadh");
        assert_eq!(f(&v.imper.sg1),  "brisim");
        assert_eq!(f(&v.imper.sg2),  "bris");
        assert_eq!(f(&v.imper.pl1),  "brisimis");
        assert_eq!(f(&v.imper.pl2),  "brisigí");
        assert_eq!(f(&v.imper.pl3),  "brisidís");
        assert_eq!(f(&v.imper.auto), "bristear");
    }

    #[test]
    fn test_bris_subj() {
        let v = Verb::from_lemma("bris", VerbConjugationClass::First);
        assert_eq!(f(&v.subj.base), "brise");
        assert_eq!(f(&v.subj.pl1),  "brisimid");
        assert_eq!(f(&v.subj.auto), "bristear");
    }

    // --- 2nd conjugation, -aigh: ceannaigh ---

    #[test]
    fn test_ceannaigh_past() {
        let v = Verb::from_lemma("ceannaigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.past.indep.base), "ceannaigh");
        assert_eq!(f(&v.past.indep.pl1),  "ceannaíomar");
        assert_eq!(f(&v.past.indep.pl3),  "ceannaíodar");
        assert_eq!(f(&v.past.indep.auto), "ceannaíodh");
    }

    #[test]
    fn test_ceannaigh_pres_cont() {
        let v = Verb::from_lemma("ceannaigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.pres_cont.indep.base), "ceannaíonn");
        assert_eq!(f(&v.pres_cont.indep.sg1),  "ceannaím");
        assert_eq!(f(&v.pres_cont.indep.pl1),  "ceannaímid");
        assert_eq!(f(&v.pres_cont.indep.auto), "ceannaítear");
    }

    #[test]
    fn test_ceannaigh_fut() {
        let v = Verb::from_lemma("ceannaigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.fut.indep.base), "ceannóidh");
        assert_eq!(f(&v.fut.indep.pl1),  "ceannóimid");
        assert_eq!(f(&v.fut.indep.auto), "ceannófar");
    }

    #[test]
    fn test_ceannaigh_cond() {
        let v = Verb::from_lemma("ceannaigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.cond.indep.base), "ceannódh");
        assert_eq!(f(&v.cond.indep.sg1),  "ceannóinn");
        assert_eq!(f(&v.cond.indep.sg2),  "ceannófá");
        assert_eq!(f(&v.cond.indep.pl1),  "ceannóimis");
        assert_eq!(f(&v.cond.indep.pl3),  "ceannóidís");
        assert_eq!(f(&v.cond.indep.auto), "ceannófaí");
    }

    #[test]
    fn test_ceannaigh_imper() {
        let v = Verb::from_lemma("ceannaigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.imper.base), "ceannaíodh");
        assert_eq!(f(&v.imper.sg2),  "ceannaigh");
        assert_eq!(f(&v.imper.pl2),  "ceannaígí");
    }

    #[test]
    fn test_ceannaigh_subj() {
        let v = Verb::from_lemma("ceannaigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.subj.base), "ceannaí");
    }

    // --- 2nd conjugation, -igh (non-aigh): bailigh ---

    #[test]
    fn test_bailigh_pres_cont() {
        let v = Verb::from_lemma("bailigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.pres_cont.indep.base), "bailíonn");
        assert_eq!(f(&v.pres_cont.indep.sg1),  "bailím");
        assert_eq!(f(&v.pres_cont.indep.pl1),  "bailímid");
        assert_eq!(f(&v.pres_cont.indep.auto), "bailítear");
    }

    #[test]
    fn test_bailigh_fut() {
        let v = Verb::from_lemma("bailigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.fut.indep.base), "baileoidh");
        assert_eq!(f(&v.fut.indep.pl1),  "baileoimid");
        assert_eq!(f(&v.fut.indep.auto), "baileofar");
    }

    #[test]
    fn test_bailigh_cond() {
        let v = Verb::from_lemma("bailigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.cond.indep.base), "baileodh");
        assert_eq!(f(&v.cond.indep.sg1),  "baileoinn");
        assert_eq!(f(&v.cond.indep.auto), "baileofaí");
    }

    // --- 2nd conjugation, syncopating: oscail ---

    #[test]
    fn test_oscail_past() {
        let v = Verb::from_lemma("oscail", VerbConjugationClass::Second);
        assert_eq!(f(&v.past.indep.base), "oscail");
        assert_eq!(f(&v.past.indep.pl1),  "osclaíomar");
        assert_eq!(f(&v.past.indep.auto), "osclaíodh");
    }

    #[test]
    fn test_oscail_pres_cont() {
        let v = Verb::from_lemma("oscail", VerbConjugationClass::Second);
        assert_eq!(f(&v.pres_cont.indep.base), "osclaíonn");
        assert_eq!(f(&v.pres_cont.indep.sg1),  "osclaím");
        assert_eq!(f(&v.pres_cont.indep.pl1),  "osclaímid");
        assert_eq!(f(&v.pres_cont.indep.auto), "osclaítear");
    }

    #[test]
    fn test_oscail_fut() {
        let v = Verb::from_lemma("oscail", VerbConjugationClass::Second);
        assert_eq!(f(&v.fut.indep.base), "osclóidh");
        assert_eq!(f(&v.fut.indep.pl1),  "osclóimid");
        assert_eq!(f(&v.fut.indep.auto), "osclófar");
    }

    #[test]
    fn test_oscail_imper() {
        let v = Verb::from_lemma("oscail", VerbConjugationClass::Second);
        assert_eq!(f(&v.imper.sg2), "oscail");
        assert_eq!(f(&v.imper.pl2), "osclaígí");
    }

    // --- get_lemma ---

    #[test]
    fn test_get_lemma() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(v.get_lemma(), "mol");

        let v = Verb::from_lemma("ceannaigh", VerbConjugationClass::Second);
        assert_eq!(v.get_lemma(), "ceannaigh");
    }

    // --- indep == dep for regular verbs ---

    #[test]
    fn test_regular_indep_eq_dep() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.base), f(&v.past.dep.base));
        assert_eq!(f(&v.fut.indep.base),  f(&v.fut.dep.base));
        assert_eq!(f(&v.cond.indep.auto), f(&v.cond.dep.auto));
    }

    // --- Relative independent forms (Python v2 addition) ---

    #[test]
    fn test_mol_rel_indep() {
        let v = Verb::from_lemma("mol", VerbConjugationClass::First);
        assert_eq!(f(&v.pres_cont.rel_indep.base), "molas");
        // molfaidh: -faidh → -idh match (not -fidh) → strip 3 + "s" = "molfas"
        assert_eq!(f(&v.fut.rel_indep.base), "molfas");
    }

    #[test]
    fn test_bris_rel_indep() {
        let v = Verb::from_lemma("bris", VerbConjugationClass::First);
        assert_eq!(f(&v.pres_cont.rel_indep.base), "briseas");
        // brisfidh → strip "fidh" → "bris" + "feas" = "brisfeas"
        assert_eq!(f(&v.fut.rel_indep.base), "brisfeas");
    }

    #[test]
    fn test_ceannaigh_rel_indep() {
        let v = Verb::from_lemma("ceannaigh", VerbConjugationClass::Second);
        // ceannaíonn → strip "nn" → "ceannaío" + "s" = "ceannaíos"
        assert_eq!(f(&v.pres_cont.rel_indep.base), "ceannaíos");
        // ceannóidh → strip "idh" → "ceannó" + "s" = "ceannós"
        assert_eq!(f(&v.fut.rel_indep.base), "ceannós");
    }

    // --- 1st conj -áil syncope (BuNaMo validated) ---

    #[test]
    fn test_sabhail_syncope() {
        let v = Verb::from_lemma("sábháil", VerbConjugationClass::First);
        // Syncopated broad stem for most forms
        assert_eq!(f(&v.past.indep.auto),       "sábháladh");
        assert_eq!(f(&v.past_cont.indep.base),  "sábháladh");
        assert_eq!(f(&v.past_cont.indep.sg1),   "sábhálainn");
        assert_eq!(f(&v.pres_cont.indep.base),  "sábhálann");
        assert_eq!(f(&v.fut.indep.base),        "sábhálfaidh");
        assert_eq!(f(&v.cond.indep.base),       "sábhálfadh");
        // Original slender stem for t-starting suffixes
        assert_eq!(f(&v.past_cont.indep.sg2),   "sábháilteá");
        assert_eq!(f(&v.past_cont.indep.auto),  "sábháiltí");
        assert_eq!(f(&v.pres_cont.indep.auto),  "sábháiltear");
    }

    // --- ae diphthong: is_slender_i in 2nd conj (C# NP.cs line 353) ---

    #[test]
    fn test_gaelaigh_broad() {
        let v = Verb::from_lemma("Gaelaigh", VerbConjugationClass::Second);
        // ae diphthong is broad → ó not eo
        assert_eq!(f(&v.fut.indep.base), "Gaelóidh");
        assert_eq!(f(&v.pres_cont.indep.base), "Gaelaíonn");
    }

    // --- Irregular verbs from embedded BuNaMo XML ---

    #[test]
    fn test_bi_pres_indep_dep() {
        let v = Verb::from_lemma("bí", VerbConjugationClass::Irregular);
        // bí has distinct indep/dep present forms
        assert_eq!(f(&v.pres.indep.base), "tá");
        assert_eq!(f(&v.pres.dep.base),   "fuil");
        assert_eq!(f(&v.pres.indep.sg1),  "táim");
        assert_eq!(f(&v.pres.dep.sg1),    "fuilim");
    }

    #[test]
    fn test_bi_past() {
        let v = Verb::from_lemma("bí", VerbConjugationClass::Irregular);
        // XML stores unlenited forms (mutations applied at sentence level)
        assert_eq!(f(&v.past.indep.base), "bí");
        assert_eq!(f(&v.past.dep.base),   "raibh");
    }

    #[test]
    fn test_bi_verbal_noun() {
        let v = Verb::from_lemma("bí", VerbConjugationClass::Irregular);
        assert_eq!(f(&v.verbal_noun), "bheith");
    }

    #[test]
    fn test_bi_rel_indep() {
        let v = Verb::from_lemma("bí", VerbConjugationClass::Irregular);
        // bí has explicit RelIndep forms in the XML
        assert_eq!(f(&v.pres.rel_indep.base), "tá");
        assert_eq!(f(&v.pres_cont.rel_indep.base), "bíos");
        assert_eq!(f(&v.fut.rel_indep.base), "beas");
    }

    #[test]
    fn test_abair_past() {
        let v = Verb::from_lemma("abair", VerbConjugationClass::Irregular);
        assert_eq!(f(&v.past.indep.base), "dúirt");
        assert_eq!(f(&v.past.indep.auto), "dúradh");
    }

    #[test]
    fn test_abair_pres_cont() {
        let v = Verb::from_lemma("abair", VerbConjugationClass::Irregular);
        assert_eq!(f(&v.pres_cont.indep.base), "deir");
        assert_eq!(f(&v.pres_cont.indep.sg1),  "deirim");
    }

    #[test]
    fn test_abair_verbal() {
        let v = Verb::from_lemma("abair", VerbConjugationClass::Irregular);
        assert_eq!(f(&v.verbal_noun), "rá");
        assert_eq!(f(&v.verbal_adjective),  "ráite");
    }

    #[test]
    fn test_faigh_fut() {
        let v = Verb::from_lemma("faigh", VerbConjugationClass::Irregular);
        assert_eq!(f(&v.fut.indep.base), "geobhaidh");
        assert_eq!(f(&v.fut.dep.base),   "faighidh");
    }

    #[test]
    fn test_dean_past() {
        let v = Verb::from_lemma("déan", VerbConjugationClass::Irregular);
        assert_eq!(f(&v.past.indep.base), "rinne");
        assert_eq!(f(&v.past.dep.base),   "dearna");
    }

    #[test]
    fn test_all_irregulars_load() {
        for lemma in IRREGULARS {
            let v = Verb::from_lemma(lemma, VerbConjugationClass::Irregular);
            assert!(!v.past.indep.base.is_empty(),
                "irregular '{}' should have past indep base", lemma);
            assert!(!v.imper.sg2.is_empty(),
                "irregular '{}' should have imperative sg2", lemma);
        }
    }

    // --- Long-vowel polysyllabic 1st-conj depalatalisation (BuNaMo validated) ---

    #[test]
    fn test_taispeain_depalatalise() {
        let v = Verb::from_lemma("taispeáin", VerbConjugationClass::First);
        // -áin: long vowel, uses broaden for all stems (taispeán-)
        assert_eq!(f(&v.past.indep.auto), "taispeánadh");
        assert_eq!(f(&v.fut.indep.base), "taispeánfaidh");
        // t-suffix: broadened stem for -áin (not -áil)
        assert_eq!(f(&v.pres_cont.indep.auto), "taispeántar");
        assert_eq!(f(&v.past_cont.indep.sg2),  "taispeántá");
        assert_eq!(f(&v.past_cont.indep.auto), "taispeántaí");
    }

    // --- Monosyllabic -igh verbs (BuNaMo validated) ---

    #[test]
    fn test_doigh_broad_root() {
        let v = Verb::from_lemma("dóigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.base),       "dóigh");
        assert_eq!(f(&v.past.indep.auto),       "dódh");
        assert_eq!(f(&v.past.indep.pl1),        "dómar");
        assert_eq!(f(&v.past_cont.indep.base),  "dódh");
        assert_eq!(f(&v.past_cont.indep.sg1),   "dóinn");
        assert_eq!(f(&v.past_cont.indep.sg2),   "dóiteá");
        assert_eq!(f(&v.past_cont.indep.auto),  "dóití");
        assert_eq!(f(&v.pres_cont.indep.base),  "dónn");
        assert_eq!(f(&v.pres_cont.indep.sg1),   "dóim");
        assert_eq!(f(&v.pres_cont.indep.auto),  "dóitear");
        assert_eq!(f(&v.fut.indep.base),        "dófaidh");
        assert_eq!(f(&v.cond.indep.base),       "dófadh");
        assert_eq!(f(&v.imper.sg2),             "dóigh");
        assert_eq!(f(&v.subj.base),             "dó");
    }

    #[test]
    fn test_leigh_slender_root() {
        let v = Verb::from_lemma("léigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "léadh");
        assert_eq!(f(&v.past.indep.pl1),        "léamar");
        assert_eq!(f(&v.past_cont.indep.base),  "léadh");
        assert_eq!(f(&v.past_cont.indep.sg1),   "léinn");
        assert_eq!(f(&v.past_cont.indep.sg2),   "léiteá");
        assert_eq!(f(&v.pres_cont.indep.base),  "léann");
        assert_eq!(f(&v.pres_cont.indep.sg1),   "léim");
        assert_eq!(f(&v.pres_cont.indep.auto),  "léitear");
        assert_eq!(f(&v.fut.indep.base),        "léifidh");
        assert_eq!(f(&v.cond.indep.base),       "léifeadh");
        assert_eq!(f(&v.subj.base),             "lé");
    }

    #[test]
    fn test_truaigh_diphthong_root() {
        let v = Verb::from_lemma("truaigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.pl1),        "truamar");
        assert_eq!(f(&v.past_cont.indep.base),  "truadh");
        assert_eq!(f(&v.past_cont.indep.sg1),   "truainn");
        assert_eq!(f(&v.past_cont.indep.sg2),   "truaiteá");
        assert_eq!(f(&v.pres_cont.indep.base),  "truann");
        assert_eq!(f(&v.pres_cont.indep.auto),  "truaitear");
        assert_eq!(f(&v.fut.indep.base),        "truafaidh");
        assert_eq!(f(&v.subj.base),             "trua");
    }

    #[test]
    fn test_bruigh_broad_root() {
        let v = Verb::from_lemma("brúigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "brúdh");
        assert_eq!(f(&v.pres_cont.indep.base),  "brúnn");
        assert_eq!(f(&v.fut.indep.base),        "brúfaidh");
        assert_eq!(f(&v.cond.indep.auto),       "brúfaí");
    }

    #[test]
    fn test_glaoigh_diphthong_root() {
        let v = Verb::from_lemma("glaoigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "glaodh");
        assert_eq!(f(&v.pres_cont.indep.base),  "glaonn");
        assert_eq!(f(&v.pres_cont.indep.auto),  "glaoitear");
        assert_eq!(f(&v.fut.indep.base),        "glaofaidh");
    }

    #[test]
    fn test_athleigh_compound_e_root() {
        let v = Verb::from_lemma("athléigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "athléadh");
        assert_eq!(f(&v.past_cont.indep.sg2),   "athléiteá");
        assert_eq!(f(&v.past_cont.indep.auto),  "athléití");
        assert_eq!(f(&v.pres_cont.indep.auto),  "athléitear");
        assert_eq!(f(&v.fut.indep.base),        "athléifidh");
        assert_eq!(f(&v.cond.indep.base),       "athléifeadh");
    }

    #[test]
    fn test_ionsaigh_compound_broad_root() {
        let v = Verb::from_lemma("ionsáigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "ionsádh");
        assert_eq!(f(&v.past_cont.indep.sg2),   "ionsáiteá");
        assert_eq!(f(&v.past_cont.indep.auto),  "ionsáití");
        assert_eq!(f(&v.pres_cont.indep.auto),  "ionsáitear");
        assert_eq!(f(&v.fut.indep.base),        "ionsáfaidh");
    }

    #[test]
    fn test_iaigh_ia_diphthong_root() {
        let v = Verb::from_lemma("iaigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),   "iatá");
        assert_eq!(f(&v.past_cont.indep.auto),  "iataí");
        assert_eq!(f(&v.pres_cont.indep.auto),  "iatar");
        assert_eq!(f(&v.fut.indep.base),        "iafaidh");
    }

    #[test]
    fn test_eisiaigh_compound_ia_root() {
        let v = Verb::from_lemma("eisiaigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),   "eisiatá");
        assert_eq!(f(&v.past_cont.indep.auto),  "eisiataí");
        assert_eq!(f(&v.pres_cont.indep.auto),  "eisiatar");
        assert_eq!(f(&v.fut.indep.base),        "eisiafaidh");
    }

    #[test]
    fn test_suigh_i_insertion() {
        let v = Verb::from_lemma("suigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "suíodh");
        assert_eq!(f(&v.past.indep.pl1),        "suíomar");
        assert_eq!(f(&v.past_cont.indep.base),  "suíodh");
        assert_eq!(f(&v.past_cont.indep.sg1),   "suínn");
        assert_eq!(f(&v.past_cont.indep.sg2),   "suiteá");
        assert_eq!(f(&v.past_cont.indep.auto),  "suití");
        assert_eq!(f(&v.pres_cont.indep.base),  "suíonn");
        assert_eq!(f(&v.pres_cont.indep.sg1),   "suím");
        assert_eq!(f(&v.pres_cont.indep.auto),  "suitear");
        assert_eq!(f(&v.fut.indep.base),        "suífidh");
        assert_eq!(f(&v.subj.base),             "suí");
    }

    #[test]
    fn test_nigh_consonant_root() {
        let v = Verb::from_lemma("nigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "níodh");
        assert_eq!(f(&v.past_cont.indep.sg2),   "niteá");
        assert_eq!(f(&v.pres_cont.indep.base),  "níonn");
        assert_eq!(f(&v.fut.indep.base),        "nífidh");
    }

    #[test]
    fn test_bligh_pure_consonant_root() {
        let v = Verb::from_lemma("bligh", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.base),  "blíodh");
        assert_eq!(f(&v.past_cont.indep.sg2),   "bliteá");
        assert_eq!(f(&v.past_cont.indep.auto),  "blití");
        assert_eq!(f(&v.fut.indep.base),        "blífidh");
    }

    #[test]
    fn test_athshuigh_compound_short_root() {
        let v = Verb::from_lemma("athshuigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "athshuíodh");
        assert_eq!(f(&v.past_cont.indep.sg2),   "athshuiteá");
        assert_eq!(f(&v.pres_cont.indep.auto),  "athshuitear");
        assert_eq!(f(&v.fut.indep.base),        "athshuífidh");
    }

    #[test]
    fn test_idirfhigh_lenited_compound() {
        let v = Verb::from_lemma("idirfhigh", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.auto),       "idirfhíodh");
        assert_eq!(f(&v.past_cont.indep.sg2),   "idirfhiteá");
        assert_eq!(f(&v.fut.indep.base),        "idirfhífidh");
    }

    #[test]
    fn test_stem_operations() {
        // Long-vowel endings (-áil, -áin): broaden just drops i, syncope over-strips
        assert_eq!(opers::broaden("sábháil"), "sábhál");
        assert_eq!(opers::syncope("sábháil"), "sábhl"); // too aggressive
        assert_eq!(opers::broaden("taispeáin"), "taispeán");
        assert_eq!(opers::syncope("taispeáin"), "taispn"); // too aggressive

        // Short-vowel endings: syncope and broaden give different stems
        assert_eq!(opers::syncope("achair"), "achr");
        assert_eq!(opers::broaden("achair"), "achar");
        assert_eq!(opers::syncope("adhain"), "adhn");
        assert_eq!(opers::broaden("adhain"), "adhan");
        assert_eq!(opers::syncope("labhair"), "labhr");
        assert_eq!(opers::broaden("labhair"), "labhar");

        // Already-broad: broaden is identity (Python/C# docstring: "passes through unchanged")
        assert_eq!(opers::broaden("achomharc"), "achomharc");
        assert_eq!(opers::broaden("adhmhol"), "adhmhol");
    }

    #[test]
    fn test_consonant_cluster_t_suffix() {
        // Stem ending in -t: absorb suffix leading t
        let v = Verb::from_lemma("alt", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),  "altá");
        assert_eq!(f(&v.past_cont.indep.auto), "altaí");
        assert_eq!(f(&v.pres_cont.indep.auto), "altar");
    }

    #[test]
    fn test_th_ending_t_suffix() {
        // Stem ending in -th: strip h, then absorb suffix t
        let v = Verb::from_lemma("caith", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),  "caiteá");
        assert_eq!(f(&v.past_cont.indep.auto), "caití");
        assert_eq!(f(&v.pres_cont.indep.auto), "caitear");
    }

    #[test]
    fn test_st_ending_t_suffix() {
        let v = Verb::from_lemma("baist", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),  "baisteá");
        assert_eq!(f(&v.past_cont.indep.auto), "baistí");
        assert_eq!(f(&v.pres_cont.indep.auto), "baistear");
    }

    #[test]
    fn test_oil_broadened_t_stem() {
        let v = Verb::from_lemma("tionóil", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),  "tionóltá");
        assert_eq!(f(&v.past_cont.indep.auto), "tionóltaí");
        assert_eq!(f(&v.pres_cont.indep.auto), "tionóltar");
    }

    #[test]
    fn test_uil_broadened_t_stem() {
        let v = Verb::from_lemma("barúil", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),  "barúltá");
        assert_eq!(f(&v.past_cont.indep.auto), "barúltaí");
    }

    #[test]
    fn test_ain_broadened_t_stem() {
        let v = Verb::from_lemma("treapáin", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),  "treapántá");
        assert_eq!(f(&v.past_cont.indep.auto), "treapántaí");
        assert_eq!(f(&v.pres_cont.indep.auto), "treapántar");
    }

    #[test]
    fn test_iomain_not_broadened() {
        let v = Verb::from_lemma("iomáin", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.base), "iomáineadh");
        assert_eq!(f(&v.past_cont.indep.sg2),  "iomáinteá");
        assert_eq!(f(&v.past_cont.indep.auto), "iomáintí");
    }

    #[test]
    fn test_annail_exception_broadened_t() {
        let v = Verb::from_lemma("annáil", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),  "annáltá");
        assert_eq!(f(&v.past_cont.indep.auto), "annáltaí");
    }

    #[test]
    fn test_argoin_broadened() {
        let v = Verb::from_lemma("argóin", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.base), "argónadh");
        assert_eq!(f(&v.past_cont.indep.sg2),  "argóntá");
        assert_eq!(f(&v.past_cont.indep.auto), "argóntaí");
    }

    #[test]
    fn test_athdháil_not_broadened() {
        let v = Verb::from_lemma("athdháil", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.base), "athdháileadh");
        assert_eq!(f(&v.fut.indep.base),        "athdháilfidh");
    }

    #[test]
    fn test_aisig_suppletive_root() {
        let v = Verb::from_lemma("aisig", VerbConjugationClass::First);
        assert_eq!(f(&v.past.indep.base),       "aisig");
        assert_eq!(f(&v.pres_cont.indep.base),  "aiseagann");
        assert_eq!(f(&v.fut.indep.base),        "aiseagfaidh");
        assert_eq!(f(&v.past_cont.indep.sg2),   "aiseagtá");
    }

    #[test]
    fn test_cadhail_root_override() {
        let v = Verb::from_lemma("cadhail", VerbConjugationClass::Second);
        assert_eq!(f(&v.pres_cont.indep.base),  "caidhlíonn");
        assert_eq!(f(&v.fut.indep.base),        "caidhleoidh");
    }

    #[test]
    fn test_cnamhair_root_override() {
        let v = Verb::from_lemma("cnámhair", VerbConjugationClass::Second);
        assert_eq!(f(&v.pres_cont.indep.base),  "cnáimhríonn");
        assert_eq!(f(&v.fut.indep.base),        "cnáimhreoidh");
    }

    #[test]
    fn test_neamhnigh_slender_override() {
        let v = Verb::from_lemma("neamhnigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.pres_cont.indep.base),  "neamhníonn");
        assert_eq!(f(&v.fut.indep.base),        "neamhneoidh");
    }

    #[test]
    fn test_graf_no_f_doubling() {
        let v = Verb::from_lemma("graf", VerbConjugationClass::First);
        assert_eq!(f(&v.fut.indep.base),  "grafaidh");
        assert_eq!(f(&v.cond.indep.base), "grafadh");
    }

    #[test]
    fn test_leigh_cond_pl1() {
        let v = Verb::from_lemma("léigh", VerbConjugationClass::First);
        assert_eq!(f(&v.cond.indep.pl1),  "léifimis");
        assert_eq!(f(&v.fut.indep.base),  "léifidh");
    }

    #[test]
    fn test_deep_syncope_bunamo() {
        let v = Verb::from_lemma("achair", VerbConjugationClass::First);
        assert_eq!(f(&v.fut.indep.base),          "acharfaidh");
        assert_eq!(f(&v.cond.indep.base),         "acharfadh");
        assert_eq!(f(&v.pres_cont.indep.auto),    "achairtear");
        assert_eq!(f(&v.past_cont.indep.sg2),     "achartá");

        let v = Verb::from_lemma("adhain", VerbConjugationClass::First);
        assert_eq!(f(&v.fut.indep.base),          "adhanfaidh");
        assert_eq!(f(&v.cond.indep.base),         "adhnfadh");
    }

    #[test]
    fn test_deighil_gnag_future() {
        let v = Verb::from_lemma("deighil", VerbConjugationClass::First);
        assert_eq!(f(&v.fut.indep.base),          "deighilfidh");
        assert_eq!(f(&v.cond.indep.base),         "deighilfeadh");
        assert_eq!(f(&v.pres_cont.indep.auto),    "deighltear");
    }

    #[test]
    fn test_buail_lemma_t_stem() {
        let v = Verb::from_lemma("búáil", VerbConjugationClass::First);
        assert_eq!(f(&v.past_cont.indep.sg2),   "búáilteá");
        assert_eq!(f(&v.pres_cont.indep.auto),  "búáiltear");
    }

    #[test]
    fn test_fead_empty_imperative() {
        let v = Verb::from_lemma("féad", VerbConjugationClass::First);
        assert!(v.imper.sg2.is_empty());
        assert!(v.imper.base.is_empty());
    }

    #[test]
    fn test_tarlaigh_past_base() {
        let v = Verb::from_lemma("tarlaigh", VerbConjugationClass::Second);
        assert_eq!(f(&v.past.indep.base), "tarla");
    }

    #[test]
    fn test_verbal_adjective_first_conj() {
        // Basic consonant endings
        assert_eq!(f(&Verb::from_lemma("glan", VerbConjugationClass::First).verbal_adjective), "glanta");
        assert_eq!(f(&Verb::from_lemma("bris", VerbConjugationClass::First).verbal_adjective), "briste");
        assert_eq!(f(&Verb::from_lemma("mol", VerbConjugationClass::First).verbal_adjective), "molta");
        assert_eq!(f(&Verb::from_lemma("gearr", VerbConjugationClass::First).verbal_adjective), "gearrtha");
        assert_eq!(f(&Verb::from_lemma("fág", VerbConjugationClass::First).verbal_adjective), "fágtha");
        assert_eq!(f(&Verb::from_lemma("ceap", VerbConjugationClass::First).verbal_adjective), "ceaptha");

        // -igh: strip -gh + te
        assert_eq!(f(&Verb::from_lemma("léigh", VerbConjugationClass::First).verbal_adjective), "léite");
        assert_eq!(f(&Verb::from_lemma("dóigh", VerbConjugationClass::First).verbal_adjective), "dóite");

        // -bh/-mh: strip + fa
        assert_eq!(f(&Verb::from_lemma("scríobh", VerbConjugationClass::First).verbal_adjective), "scríofa");
        assert_eq!(f(&Verb::from_lemma("snámh", VerbConjugationClass::First).verbal_adjective), "snáfa");

        // -th: strip + ta/te
        assert_eq!(f(&Verb::from_lemma("caith", VerbConjugationClass::First).verbal_adjective), "caite");
        assert_eq!(f(&Verb::from_lemma("ith", VerbConjugationClass::First).verbal_adjective), "ite");

        // Broadened stems
        assert_eq!(f(&Verb::from_lemma("cuir", VerbConjugationClass::First).verbal_adjective), "curtha");
        assert_eq!(f(&Verb::from_lemma("goin", VerbConjugationClass::First).verbal_adjective), "gonta");
        assert_eq!(f(&Verb::from_lemma("taispeáin", VerbConjugationClass::First).verbal_adjective), "taispeánta");

        // No double-t
        assert_eq!(f(&Verb::from_lemma("alt", VerbConjugationClass::First).verbal_adjective), "alta");
    }

    #[test]
    fn test_verbal_adjective_second_conj() {
        // -igh: strip -gh + the
        assert_eq!(f(&Verb::from_lemma("salaigh", VerbConjugationClass::Second).verbal_adjective), "salaithe");
        assert_eq!(f(&Verb::from_lemma("coinnigh", VerbConjugationClass::Second).verbal_adjective), "coinnithe");

        // -il: stem + te
        assert_eq!(f(&Verb::from_lemma("oscail", VerbConjugationClass::Second).verbal_adjective), "oscailte");

        // -in: broaden + ta
        assert_eq!(f(&Verb::from_lemma("imir", VerbConjugationClass::Second).verbal_adjective), "imeartha");

        // -im: broaden + tha
        assert_eq!(f(&Verb::from_lemma("foghlaim", VerbConjugationClass::Second).verbal_adjective), "foghlamtha");
    }
}
