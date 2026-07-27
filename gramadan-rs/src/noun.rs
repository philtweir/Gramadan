use crate::features::{Form, FormPlGen, FormSg, Gender};
use crate::opers;
use crate::plural_info::PluralInfo;
use crate::singular_info::{self, SingularInfo};

use regex::Regex;

// ---- Noun struct ----

/// An Irish noun with its full paradigm.
#[derive(Debug, Clone)]
pub struct Noun {
    pub sg_nom: Vec<FormSg>,
    pub sg_gen: Vec<FormSg>,
    pub sg_voc: Vec<Form>,
    pub sg_dat: Vec<Form>,
    pub pl_nom: Vec<Form>,
    pub pl_gen: Vec<FormPlGen>,
    pub pl_voc: Vec<Form>,
    pub count: Vec<Form>,
    pub declension: i8, // 0=unknown, 1-5=standard, -1=irregular
    pub is_proper: bool,
    pub is_immutable: bool,
    pub is_definite: bool,
    pub allow_articled_genitive: bool,
    pub disambig: String,
}

impl Noun {
    pub fn new() -> Self {
        Self {
            sg_nom: Vec::new(),
            sg_gen: Vec::new(),
            sg_voc: Vec::new(),
            sg_dat: Vec::new(),
            pl_nom: Vec::new(),
            pl_gen: Vec::new(),
            pl_voc: Vec::new(),
            count: Vec::new(),
            declension: 0,
            is_proper: false,
            is_immutable: false,
            is_definite: false,
            allow_articled_genitive: true,
            disambig: String::new(),
        }
    }

    /// Create a noun from singular and plural info.
    pub fn create_from_info(
        si: &SingularInfo,
        pi: Option<&PluralInfo>,
        declension: i8,
    ) -> Self {
        let mut noun = Self::new();
        noun.declension = declension;

        for f in &si.nominative {
            noun.sg_nom.push(FormSg::new(&f.value, si.gender));
        }
        for f in &si.genitive {
            noun.sg_gen.push(FormSg::new(&f.value, si.gender));
        }
        for f in &si.vocative {
            noun.sg_voc.push(Form::new(&f.value));
        }
        for f in &si.dative {
            noun.sg_dat.push(Form::new(&f.value));
        }

        if let Some(pi) = pi {
            for f in &pi.nominative {
                noun.pl_nom.push(Form::new(&f.value));
            }
            for f in &pi.genitive {
                noun.pl_gen.push(FormPlGen::new(&f.value, pi.strength));
            }
            for f in &pi.vocative {
                noun.pl_voc.push(Form::new(&f.value));
            }
        }

        if noun.sg_dat.is_empty() {
            for f in &noun.sg_nom {
                noun.sg_dat.push(Form::new(&f.value));
            }
        }

        noun
    }

    pub fn from_lemma(lemma: &str, gender: Gender) -> Self {
        let mut noun = Self::new();
        noun.sg_nom.push(FormSg::new(lemma, gender));
        noun
    }

    pub fn from_lemma_gen(lemma: &str, genitive: &str, gender: Gender) -> Self {
        let mut noun = Self::new();
        noun.sg_nom.push(FormSg::new(lemma, gender));
        noun.sg_gen.push(FormSg::new(genitive, gender));
        noun
    }

    pub fn get_lemma(&self) -> &str {
        self.sg_nom.first().map(|f| f.value.as_str()).unwrap_or("")
    }

    pub fn get_gender(&self) -> Gender {
        self.sg_nom.first().map(|f| f.gender).unwrap_or(Gender::Masc)
    }
}

// ---- Declension type ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declension {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Irregular,
}

impl Declension {
    pub fn as_i8(&self) -> i8 {
        match self {
            Declension::First => 1,
            Declension::Second => 2,
            Declension::Third => 3,
            Declension::Fourth => 4,
            Declension::Fifth => 5,
            Declension::Irregular => -1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            1 => Some(Declension::First),
            2 => Some(Declension::Second),
            3 => Some(Declension::Third),
            4 => Some(Declension::Fourth),
            5 => Some(Declension::Fifth),
            -1 => Some(Declension::Irregular),
            _ => None,
        }
    }
}

// ============================================================
// Exception dictionaries (ported from Python noun_declensions.py
// and noun_nualeargais.py)
// ============================================================

/// Words with multiple declensions depending on meaning (matched by genitive).
const MULTIPLE_WORDS: &[(&str, &[(i8, &str)])] = &[
    ("sail", &[(5, "saileach"), (2, "saile")]),
    ("cian", &[(1, "cian"), (2, "céine")]),
];

/// Irregular declension nouns — genitive is completely unpredictable.
/// (lemma -> (declension, genitive))
const FULLY_IRREGULAR: &[(&str, (i8, &str))] = &[
    ("laghad", (1, "laghad")),
    ("anachain", (3, "anachaine")),
    ("leann", (1, "leanna")),
    ("bunáite", (2, "bunáite")),
    ("troitheán", (4, "troitheáin")),
    ("onnmhaireoir", (4, "onnmhaireora")),
    ("gínéiceolaíocht", (3, "gínéiceolaíocht")),
    ("réamhghlacan", (2, "réamhghlacana")),
    ("ardaicme", (2, "ardaicme")),
    ("fobhóthar", (1, "fobhóthar")),
    ("righneáil", (2, "righneála")),
    ("catacóm", (1, "catacóma")),
    ("spásrás", (1, "spásrása")),
    ("cúblálaí", (3, "cúblálaí")),
    ("slogadh", (1, "slogtha")),
    ("cionroinnt", (2, "cionranna")),
    // First declension irregulars
    ("cian", (1, "cian")),
    ("fuineadh", (1, "fuinte")),
    ("muineál", (1, "muiníl")),
    ("ainchleachtadh", (1, "ainchleachta")),
    ("bonnbhualadh", (1, "bonnbhuailte")),
    ("díraonadh", (1, "díraonta")),
    ("leabharchoimeád", (1, "leabharchoimeádta")),
    ("domhainfhriochtán", (1, "domhanfhriochtáin")),
    ("Malaech", (1, "Malaeich")),
    ("laoch", (1, "laoich")),
    ("caoch", (1, "caoich")),
    ("éicealaoch", (1, "éicealaoich")),
    ("crannlaoch", (1, "crannlaoich")),
    ("dobhareach", (1, "dobhareich")),
    ("each", (1, "eich")),
    ("stóch", (1, "stóich")),
    ("cóch", (1, "cóich")),
    ("fíoch", (1, "fích")),
    ("ochtach", (1, "ochtaí")),
    ("bearach", (1, "bearaí")),
    ("Gael", (1, "Gaeil")),
    ("taghd", (1, "taghaid")),
    // Second declension irregulars
    ("fidil", (2, "fidle")),
    ("scian", (2, "scine")),
    ("sliabh", (2, "sléibhe")),
    ("bansliabh", (2, "bansléibhe")),
    ("droimshliabh", (2, "droimshléibhe")),
    ("blocshliabh", (2, "blocsléibhe")),
    ("loilíoch", (2, "loilí")),
    // Third declension irregulars
    ("prios", (3, "priosa")),
    ("goid", (3, "gada")),
    ("conradh", (3, "conartha")),
    ("cumhachtroinnt", (3, "cumhachtroinnte")),
    ("bunmhúinteoir", (3, "bunmhuinteora")),
    ("cion", (3, "ciona")),
    ("cosaint", (3, "cosanta")),
    ("dioc", (3, "dioca")),
    ("siorc", (3, "siorca")),
    ("giolc", (3, "giolca")),
    ("triuch", (3, "treacha")),
    ("miocht", (3, "miochta")),
    ("toirbhirt", (3, "toirbhearta")),
    ("iarmhairt", (3, "iarmharta")),
    ("mionn", (3, "mionna")),
    ("méadail", (3, "méadla")),
    ("iarraidh", (3, "iarrata")),
    ("muir", (3, "mara")),
    ("mallmhuir", (3, "mallmhara")),
    ("seachaint", (3, "seachanta")),
    // Fourth declension irregulars
    ("dháréag", (4, "dáréag")),
    ("ionsaí", (4, "ionsaithe")),
    ("spré", (4, "spréite")),
    ("dreo", (4, "dreoite")),
    ("araí", (4, "araíon")),
    ("coinnealbhá", (4, "coinnealbháite")),
    // Teach family 1st declension (irregular genitive -tí)
    ("díonteach", (1, "díontí")),
    ("craobhtheach", (1, "craobhthí")),
    // fiach: without_iai but also irregular slenderization
    ("fiach", (1, "féich")),
    // Fifth declension irregulars
    ("namhaid", (5, "namhad")),
    ("bráid", (5, "brád")),
    ("Nollaig", (5, "Nollag")),
];

/// Words in the "Irregular Declension" (6th declension category).
/// Used by the base and full guessers. The simple guesser overrides with empty.
const IRREGULAR_DECLENSION_FULL: &[(&str, &str)] = &[
    ("bean", "mná"),
    ("deirfiúr", "deirfear"),
    ("siúr", "siúrach"),
    ("dia", "dé"),
    ("lá", "lae"),
    ("leaba", "leapa"),
    ("mí", "míosa"),
    ("olann", "olla"),
    ("talamh", "talún"),
    ("ó", "uí"),
];

/// Base class also includes deoch; full guesser does not (it overrides).
const IRREGULAR_DECLENSION_BASE: &[(&str, &str)] = &[
    ("bean", "mná"),
    ("deirfiúr", "deirfear"),
    ("deoch", "dí"),
    ("siúr", "siúrach"),
    ("dia", "dé"),
    ("lá", "lae"),
    ("leaba", "leapa"),
    ("mí", "míosa"),
    ("olann", "olla"),
    ("talamh", "talún"),
    ("ó", "uí"),
];

/// Family group — 5th declension.
const FAMILY: &[&str] = &[
    "athair",
    "seanathair",
    "bráthair",
    "comhbhráthair",
    "deartháir",
    "leathdheartháir",
    "seanmháthair",
    "ríonmháthair",
    "máthair",
];

/// Teach family — irregular declension.
const TEACH_FAMILY: &[&str] = &[
    "cloigtheach", "clubtheach", "cléirtheach", "coirmtheach", "cruinnteach",
    "cúlteach", "dairtheach", "fiailteach", "fleiteach", "fortheach",
    "longtheach", "mainteach", "máthairtheach", "plódteach", "proinnteach",
    "rítheach", "teach", "túrtheach", "urtheach", "íolteach",
];

/// Teach family derivatives that BuNaMo puts in 1st declension.
const TEACH_FAMILY_1ST: &[&str] = &["craobhtheach", "díonteach"];

/// Possible loanwords that take 4th declension (genitiveless) when consonant-ending.
const POSSIBLE_LOANWORDS_GENITIVELESS: &[&str] = &[
    "ailím", "aintín", "apacailipsis", "basal", "beirilliam", "biogóid",
    "blitz", "bob", "bus", "call", "carst", "ceaig", "ceilvin",
    "ceirbheacs", "cic", "clinic", "club", "Comaoineach", "Críost", "cróch",
    "dabht", "daid", "dosaen", "fabht", "fastaím", "fean", "franc", "fuist",
    "geab", "geiréiniam", "gigiheirts", "gild", "giúistís", "hap",
    "héileapad", "hiopstar", "jab", "jíp", "jóc", "Laos", "leaid",
    "Madagascar", "maidhc", "Máirt", "matrarc", "meigiheirts", "meireang",
    "Meisias", "miorr", "monarc", "mosc", "nanashoicind", "Oiríon", "Óman",
    "pas", "patrarc", "péas", "Peintiteoch", "Pilib", "pioc", "píoláf",
    "pionsail", "píoráid", "plean", "pontaif", "pram", "punc", "raidhfil",
    "raifil", "rascail", "rúbarb", "rum", "sáirsint", "seaimpéin", "seic",
    "seilf", "seirbhísiú", "seit", "síc", "siút", "slám", "soicind",
    "speár", "stop", "stróc", "stuif", "tanc", "tic", "tinsil", "tobac",
    "traidhfil", "tram", "uncail", "vác", "veain", "Véineas", "zip",
    "rotharchic", "saorchic", "mionbhus",
];

/// Dec 1: words with irregular slenderization targets.
const IRREGULARLY_PALATALIZED_1ST: &[(&str, &str)] = &[
    ("mac", "i"), ("garmhac", "i"),
    ("bligeard", "eai"), ("seac", "eai"),
    ("earc", "ei"), ("gaistreintríteas", "ei"), ("dearg", "ei"),
    ("meadhg", "ei"), ("corcairdhearg", "ei"), ("infridhearg", "ei"),
    ("Eiritréach", "éai"),
];

/// Dec 1: words where `with_iai` should be false.
const WITHOUT_IAI: &[&str] = &["cliabh", "fiach", "fial", "giall"];

/// Dec 2: monosyllabic words that use "i" target, not "ei".
const MONOSYLLABIC_I_2ND: &[&str] = &["cearc", "beanne", "beann"];

/// Dec 2: polysyllabic words that use "ei" target.
const POLYSYLLABIC_EI_2ND: &[&str] = &[
    "meadar", "maoildearg", "taibhdhearc", "seamair", "díthreabh",
    "aershreabh", "bláthfhleasc", "bonnleac", "cráinbheach",
];

/// Dec 3: words where auto-syncope for -ain/-ail/-air should NOT apply.
const UNSYNCOPATED_3RD: &[&str] = &[
    "athbhliain", "bliain", "bonnbhliain", "idirbhliain", "scoilbhliain", "solasbhliain",
    "argain", "cluain", "dúnorgain", "foluain", "fothain",
    "féachaint", "súilfhéachaint", "tuargaint",
    "marthain",
    "cantain", "cianrochtain", "comhriachtain", "dámhachtain", "fionnachtain",
    "mainneachtain", "rochtain",
];

// ---- Helper ----

fn re_ends(lemma: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| {
        Regex::new(&format!("{}$", p)).unwrap().is_match(lemma)
    })
}

fn lookup_fully_irregular(lemma: &str) -> Option<(i8, &'static str)> {
    FULLY_IRREGULAR.iter()
        .find(|(l, _)| *l == lemma)
        .map(|(_, v)| *v)
}

fn lookup_multiple_words(lemma: &str) -> Option<&'static [(i8, &'static str)]> {
    MULTIPLE_WORDS.iter()
        .find(|(l, _)| *l == lemma)
        .map(|(_, v)| *v)
}

fn in_irregular_declension(lemma: &str, table: &[(&str, &str)]) -> bool {
    table.iter().any(|(l, _)| *l == lemma)
}

fn in_list(lemma: &str, list: &[&str]) -> bool {
    list.contains(&lemma)
}

// ============================================================
// Declension guessing — Simple (NualeargaisNounDeclensionGuesser)
// ============================================================

/// Guess noun declension from lemma + gender only (no genitive needed).
///
/// Matches the Python `NualeargaisNounDeclensionGuesser`.
/// Check order: 5th (always false) → 4th → 3rd → 2nd → 1st, default 3rd.
pub fn guess_declension(lemma: &str, gender: Gender) -> Declension {
    // Simple guesser has IRREGULAR_DECLENSION = {} (empty override), so skip it.

    // MULTIPLE_WORDS (inherited from base)
    // Note: the simple guesser's guess() method does check these, but without
    // a known genitive we can't disambiguate. Skip for lemma-only guessing.

    // FULLY_IRREGULAR (inherited from base)
    if let Some((dec, _)) = lookup_fully_irregular(lemma) {
        return Declension::from_i8(dec).unwrap_or(Declension::Irregular);
    }

    // IRREGULAR_GROUPS: empty in simple guesser. Skip.

    // IRREGULAR_INCLUSION for simple guesser: im→2, sliabh→2
    // These are applied inside the checks loop, not as immediate returns.
    // We handle them inline below.
    let irregular_inclusion: Option<i8> = match lemma {
        "im" => Some(2),
        "sliabh" => Some(2),
        _ => None,
    };

    // Check order: 5th → 4th → 3rd → 2nd → 1st
    // (5th always returns false in simple guesser)
    let checks: &[(i8, fn(&str, Gender) -> bool)] = &[
        (5, is_fifth_simple),
        (4, is_fourth_simple),
        (3, is_third_simple),
        (2, is_second_simple),
        (1, is_first_simple),
    ];

    for &(dec, check) in checks {
        if irregular_inclusion == Some(dec) || (irregular_inclusion.is_none() && check(lemma, gender)) {
            return Declension::from_i8(dec).unwrap_or(Declension::Third);
        }
    }

    Declension::Third
}

fn is_fifth_simple(_lemma: &str, _gender: Gender) -> bool {
    false
}

fn is_fourth_simple(lemma: &str, gender: Gender) -> bool {
    if gender == Gender::Fem && re_ends(lemma, &["[eí]"]) {
        return true;
    }
    if gender == Gender::Masc && re_ends(lemma, &[&format!("[{}]", opers::VOWELS), "ín"]) {
        return true;
    }
    false
}

fn is_third_simple(lemma: &str, gender: Gender) -> bool {
    if gender == Gender::Fem && re_ends(lemma, &["áil", "úil", "ail", "úint", "cht", "irt"]) {
        return true;
    }
    if gender == Gender::Masc && re_ends(lemma, &["éir", "eoir", "óir", "úir"]) {
        return true;
    }
    false
}

fn is_second_simple(lemma: &str, gender: Gender) -> bool {
    if gender != Gender::Fem {
        return false;
    }
    if opers::is_slender(lemma) {
        return true;
    }
    if re_ends(lemma, &["eog", "óg", "lann"]) {
        return true;
    }
    false
}

fn is_first_simple(lemma: &str, gender: Gender) -> bool {
    gender == Gender::Masc && !opers::is_slender(lemma)
}

// ============================================================
// Declension guessing — Full (NualeargaisFullNounDeclensionGuesser)
// ============================================================

/// Guess noun declension with the "Full" guesser variant.
///
/// Matches the Python `NualeargaisFullNounDeclensionGuesser`.
/// Includes irregular word lists, TEACH_FAMILY, loanword handling.
/// Check order: 4th → 1st → 2nd → 5th, default 3rd.
pub fn guess_declension_full(lemma: &str, gender: Gender) -> Declension {
    // IRREGULAR_DECLENSION (full guesser override — does not include deoch)
    if in_irregular_declension(lemma, IRREGULAR_DECLENSION_FULL) {
        return Declension::Irregular;
    }

    // MULTIPLE_WORDS (inherited)
    // Without genitive, we can't disambiguate — skip.

    // FULLY_IRREGULAR (inherited from base)
    if let Some((dec, _)) = lookup_fully_irregular(lemma) {
        return Declension::from_i8(dec).unwrap_or(Declension::Irregular);
    }

    // IRREGULAR_GROUPS: FAMILY → 5th
    if in_list(lemma, FAMILY) {
        return Declension::Fifth;
    }

    // TEACH_FAMILY → Irregular
    if in_list(lemma, TEACH_FAMILY) {
        return Declension::Irregular;
    }
    // TEACH_FAMILY_1ST → First
    if in_list(lemma, TEACH_FAMILY_1ST) {
        return Declension::First;
    }

    // IRREGULAR_INCLUSION for full guesser: im→2, sliabh→2, teach→2, ainm→4, lucht→4
    // Applied inside the checks loop.
    // Note: teach is already caught by TEACH_FAMILY above (returns Irregular).
    let irregular_inclusion: Option<i8> = match lemma {
        "im" | "sliabh" | "teach" => Some(2),
        "ainm" | "lucht" => Some(4),
        _ => None,
    };

    // Check order: 4th → 1st → 2nd → 5th
    let checks: &[(i8, fn(&str, Gender) -> bool)] = &[
        (4, is_fourth_full),
        (1, is_first_full),
        (2, is_second_full),
        (5, is_fifth_full),
    ];

    for &(dec, check) in checks {
        if irregular_inclusion == Some(dec) || (irregular_inclusion.is_none() && check(lemma, gender)) {
            return Declension::from_i8(dec).unwrap_or(Declension::Third);
        }
    }

    Declension::Third
}

fn is_fourth_full(lemma: &str, gender: Gender) -> bool {
    // Loanwords with consonant ending → 4th regardless of gender
    if in_list(lemma, POSSIBLE_LOANWORDS_GENITIVELESS) {
        if let Some(last) = lemma.chars().last() {
            if opers::CONSONANTS.contains(last.to_lowercase().next().unwrap_or(last)) {
                return true;
            }
        }
    }

    // Fem never gets 4th in full guesser
    if gender == Gender::Fem {
        return false;
    }

    // Masc + vowel/ín ending
    if re_ends(lemma, &[&format!("[{}]", opers::VOWELS), "ín"]) {
        return true;
    }

    false
}

fn is_first_full(lemma: &str, gender: Gender) -> bool {
    gender == Gender::Masc && !opers::is_slender(lemma)
}

fn is_second_full(lemma: &str, gender: Gender) -> bool {
    if gender != Gender::Fem {
        return false;
    }
    if let Some(last) = lemma.chars().last() {
        if opers::VOWELS.contains(last) {
            return false;
        }
    }
    true
}

fn is_fifth_full(lemma: &str, gender: Gender) -> bool {
    if gender != Gender::Fem {
        return false;
    }
    if opers::is_slender(lemma) {
        return true;
    }
    if let Some(last) = lemma.chars().last() {
        if opers::VOWELS.contains(last) {
            return true;
        }
    }
    false
}

// ============================================================
// Genitive generation — strategy selection per declension
// ============================================================

/// Generate the genitive form for a noun given lemma, gender, and declension.
///
/// Uses the appropriate SingularInfo strategy (or FULLY_IRREGULAR lookup)
/// to produce the genitive. Returns None for unknown declensions.
pub fn generate_genitive(lemma: &str, gender: Gender, declension: i8) -> Option<String> {
    // FULLY_IRREGULAR overrides everything
    if let Some((dec, gen)) = lookup_fully_irregular(lemma) {
        if dec == declension {
            return Some(gen.to_string());
        }
    }

    // IRREGULAR_DECLENSION — these have known genitives
    for &(l, gen) in IRREGULAR_DECLENSION_BASE {
        if l == lemma {
            return Some(gen.to_string());
        }
    }

    match declension {
        1 => Some(generate_genitive_1st(lemma, gender)),
        2 => Some(generate_genitive_2nd(lemma, gender)),
        3 => Some(generate_genitive_3rd(lemma, gender)),
        4 => Some(generate_genitive_4th(lemma, gender)),
        5 => Some(generate_genitive_5th(lemma, gender)),
        _ => None,
    }
}

/// 1st declension: SingularInfoC with optional irregular targets and with_iai control.
fn generate_genitive_1st(lemma: &str, gender: Gender) -> String {
    let target = IRREGULARLY_PALATALIZED_1ST.iter()
        .find(|(l, _)| *l == lemma)
        .map(|(_, t)| *t)
        .unwrap_or("");

    let with_iai = !in_list(lemma, WITHOUT_IAI);

    let si = singular_info::singular_info_c(lemma, gender, target, with_iai);
    si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string())
}

/// 2nd declension: SingularInfoE by default, SingularInfoC for polysyllabic fem -ach/-each.
///
/// Monosyllabic -ach words (cuach, beach, creach, etc.) use SingularInfoE (gen: -aiche/-eiche).
/// Polysyllabic -ach words (bolgach, óinseach, etc.) use SingularInfoC (gen: -aí/-í).
fn generate_genitive_2nd(lemma: &str, gender: Gender) -> String {
    let ei_target = if in_list(lemma, POLYSYLLABIC_EI_2ND) {
        "ei"
    } else {
        ""
    };

    let use_mono_ei = !in_list(lemma, MONOSYLLABIC_I_2ND);

    // Polysyllabic fem -ach/-each: use SingularInfoC (ch→gh, slenderize, fem -igh→-í)
    if gender == Gender::Fem
        && (lemma.ends_with("ach") || lemma.ends_with("each"))
        && opers::polysyllabic(lemma)
    {
        let si = singular_info::singular_info_c(lemma, gender, ei_target, false);
        return si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string());
    }

    // Default: SingularInfoE without syncope
    let si = singular_info::singular_info_e(lemma, gender, false, false, ei_target, use_mono_ei);
    si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string())
}

/// 3rd declension: SingularInfoA with conditional auto-syncope.
fn generate_genitive_3rd(lemma: &str, gender: Gender) -> String {
    let with_syncopated_ai = !in_list(lemma, UNSYNCOPATED_3RD);

    let si = singular_info::singular_info_a(lemma, gender, false, "", with_syncopated_ai);
    si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string())
}

/// 4th declension: gen = nom.
///
/// Some masc -ú nouns in BuNaMo have gen≠nom (verbal nouns, arguably 2nd decl behaviour),
/// but the standard 4th declension rule is gen=nom. BuNaMo composition provides the
/// exact paradigm for those cases; this generator handles the remaining ones.
fn generate_genitive_4th(lemma: &str, _gender: Gender) -> String {
    lemma.to_string()
}

/// 5th declension: branch on gender + ending.
/// Fem consonant (r/l/n): SingularInfoAX or EAX
/// Fem vowel: SingularInfoN
/// Masc vowel: SingularInfoD
/// Otherwise: SingularInfoL (broaden)
fn generate_genitive_5th(lemma: &str, gender: Gender) -> String {
    let last_char = match lemma.chars().last() {
        Some(c) => c,
        None => return lemma.to_string(),
    };

    if gender == Gender::Fem {
        if matches!(last_char, 'r' | 'l' | 'n') {
            // Fem consonant-ending: use heuristics to pick AX vs EAX
            return generate_genitive_5th_fem_consonant(lemma, gender);
        }
        if opers::VOWELS.contains(last_char) {
            // Fem vowel-ending: append -n/-an
            let si = singular_info::singular_info_n(lemma, gender);
            return si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string());
        }
    }

    if gender == Gender::Masc && opers::VOWELS.contains(last_char) {
        // Masc vowel-ending: append -d/-ad
        let si = singular_info::singular_info_d(lemma, gender);
        return si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string());
    }

    // Default: broaden
    let si = singular_info::singular_info_l(lemma, gender, "");
    si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string())
}

/// 5th declension fem nouns ending in r/l/n: pick between AX (broaden+-ach),
/// EAX (slenderize+-each), and L (plain broadening).
///
/// Heuristics ported from the Python `_is_fifth` in noun_declensions.py.
fn generate_genitive_5th_fem_consonant(lemma: &str, gender: Gender) -> String {
    // Endings that indicate AX/EAX with syncope (polysyllabic words with
    // specific patterns from Python lines 571-573)
    let ax_word_endings = [
        "thir", "mhir", "eoir", "athair", "ochair", "bhair", "eorainn",
    ];
    let ax_substring_patterns = ["riai", "tiúi"];

    let has_ax_ending = ax_word_endings.iter().any(|e| lemma.ends_with(e));
    let has_ax_pattern = ax_substring_patterns.iter().any(|p| lemma.contains(p));

    if has_ax_ending || has_ax_pattern {
        let do_syncope = opers::polysyllabic(lemma);
        let si = singular_info::singular_info_ax(lemma, gender, do_syncope, "");
        return si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string());
    }

    // Endings that indicate plain broadening (SingularInfoL) — from Python lines 576-579.
    // These are words where the penultimate vowel is long/specific.
    let broadening_endings = ["eoil", "coil", "ain", "ill"];

    // Check penultimate vowel cluster for patterns that prefer L
    let before_last: String = {
        let chars: Vec<char> = lemma.chars().collect();
        if chars.len() >= 2 {
            chars[..chars.len() - 1].iter().collect()
        } else {
            String::new()
        }
    };
    let broadening_vowel_patterns = ["í", "úi", "ói", "éi", "ái"];

    let has_broadening_ending = broadening_endings.iter().any(|e| lemma.ends_with(e));
    let has_broadening_vowel = broadening_vowel_patterns.iter().any(|p| before_last.ends_with(p));

    if has_broadening_ending || has_broadening_vowel {
        let si = singular_info::singular_info_l(lemma, gender, "");
        return si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string());
    }

    // Default for fem r/l/n: AX (majority case per Python line 581)
    // Use syncope only for polysyllabic words to avoid mangling short words
    let do_syncope = opers::polysyllabic(lemma);
    let si = singular_info::singular_info_ax(lemma, gender, do_syncope, "");
    si.genitive.first().map(|f| f.value.clone()).unwrap_or_else(|| lemma.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_first_decl() {
        assert_eq!(guess_declension("bád", Gender::Masc), Declension::First);
        assert_eq!(guess_declension("fear", Gender::Masc), Declension::First);
    }

    #[test]
    fn test_guess_second_decl() {
        assert_eq!(guess_declension("bróg", Gender::Fem), Declension::Second);
        assert_eq!(guess_declension("fuinneog", Gender::Fem), Declension::Second);
    }

    #[test]
    fn test_guess_third_decl() {
        assert_eq!(guess_declension("beannacht", Gender::Fem), Declension::Third);
        assert_eq!(guess_declension("dochtúir", Gender::Masc), Declension::Third);
    }

    #[test]
    fn test_guess_fourth_decl() {
        assert_eq!(guess_declension("bainne", Gender::Masc), Declension::Fourth);
    }

    #[test]
    fn test_guess_full_irregular() {
        assert_eq!(guess_declension_full("bean", Gender::Fem), Declension::Irregular);
        assert_eq!(guess_declension_full("teach", Gender::Masc), Declension::Irregular);
    }

    #[test]
    fn test_guess_full_family() {
        assert_eq!(guess_declension_full("athair", Gender::Masc), Declension::Fifth);
        assert_eq!(guess_declension_full("seanathair", Gender::Masc), Declension::Fifth);
        assert_eq!(guess_declension_full("máthair", Gender::Fem), Declension::Fifth);
    }

    #[test]
    fn test_fully_irregular_lookup() {
        assert_eq!(guess_declension("laoch", Gender::Masc), Declension::First);
        assert_eq!(guess_declension("scian", Gender::Fem), Declension::Second);
    }

    #[test]
    fn test_genitive_1st_regular() {
        assert_eq!(generate_genitive("bád", Gender::Masc, 1).unwrap(), "báid");
    }

    #[test]
    fn test_genitive_1st_irregular_target() {
        assert_eq!(generate_genitive("mac", Gender::Masc, 1).unwrap(), "mic");
    }

    #[test]
    fn test_genitive_2nd_ach() {
        // Fem -ach → -aí via SingularInfoC
        assert_eq!(generate_genitive("bolgach", Gender::Fem, 2).unwrap(), "bolgaí");
    }

    #[test]
    fn test_genitive_4th() {
        // 4th declension: gen = nom
        assert_eq!(generate_genitive("scrúdú", Gender::Masc, 4).unwrap(), "scrúdú");
        assert_eq!(generate_genitive("bainne", Gender::Masc, 4).unwrap(), "bainne");
    }

    #[test]
    fn test_genitive_5th_fem_vowel() {
        // Fem vowel → append -n
        assert_eq!(generate_genitive("caora", Gender::Fem, 5).unwrap(), "caoran");
    }

    #[test]
    fn test_genitive_5th_masc_vowel() {
        // Masc vowel → append -d
        assert_eq!(generate_genitive("cara", Gender::Masc, 5).unwrap(), "carad");
    }

    #[test]
    fn test_genitive_irregular_declension() {
        assert_eq!(generate_genitive("bean", Gender::Fem, 0).unwrap(), "mná");
    }
}
