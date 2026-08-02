use std::collections::HashMap;

use crate::features::{Form, FormPlGen, FormSg, Gender};
use crate::opers;
use crate::plural_info::{self, PluralInfo};
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
// Compound decomposition — LemmaDb
// ============================================================

/// A database of known lemmas for compound decomposition.
///
/// When the morphological rules can't determine declension, we check
/// whether the word is a compound whose head (final element) is a known
/// word — and if so, inherit its declension. Irish compounds take their
/// gender and declension from the head word, and the join point is often
/// lenited (e.g. `aol` + `cloch` → `aolchloch`).
#[derive(Debug, Clone)]
pub struct LemmaDb {
    /// Maps lemma → (declension, gender)
    entries: HashMap<String, (i8, Gender)>,
}

/// Lenition pairs: mutated form → original consonant.
const DEMUT_PAIRS: &[(&str, &str)] = &[
    ("bh", "b"), ("ch", "c"), ("dh", "d"), ("fh", "f"),
    ("gh", "g"), ("mh", "m"), ("ph", "p"), ("sh", "s"), ("th", "t"),
];

/// Words whose declension is inconsistent between standalone and compound
/// use in BuNaMo. Excluded from compound head matching.
const COMPOUND_HEAD_BLACKLIST: &[&str] = &["beart"];

impl LemmaDb {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Build from an iterator of (lemma, declension, gender).
    pub fn from_iter(iter: impl IntoIterator<Item = (String, i8, Gender)>) -> Self {
        let mut db = Self::new();
        for (lemma, dec, gender) in iter {
            db.entries.insert(lemma, (dec, gender));
        }
        db
    }

    pub fn insert(&mut self, lemma: String, declension: i8, gender: Gender) {
        self.entries.insert(lemma, (declension, gender));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up a lemma's declension directly.
    pub fn lookup(&self, lemma: &str) -> Option<i8> {
        self.entries.get(lemma).map(|(dec, _)| *dec)
    }

    /// Look up a lemma's declension and gender.
    pub fn lookup_full(&self, lemma: &str) -> Option<(i8, Gender)> {
        self.entries.get(lemma).copied()
    }

    /// Find the compound head of a word.
    ///
    /// Three strategies, in order of reliability:
    /// 1. **Hyphen split**: `ard-deoch` → `deoch`
    /// 2. **Lenition detection**: `aolchloch` → `cloch` (demutate at join)
    /// 3. **Direct suffix**: `buntarraingt` → `tarraingt` (for long known
    ///    heads where the join doesn't trigger lenition)
    ///
    /// All strategies require the head's gender to match `required_gender`.
    /// Returns the longest matching head's declension.
    pub fn find_compound_head(&self, lemma: &str, required_gender: Gender) -> Option<i8> {
        let mut best: Option<(i8, usize)> = None; // (declension, head_len)

        // Strategy 1: hyphen split — most reliable signal
        if let Some(hyphen_pos) = lemma.rfind('-') {
            let tail = &lemma[hyphen_pos + 1..];
            if tail.len() >= 3 {
                // Try direct match
                if let Some(&(dec, gender)) = self.entries.get(tail) {
                    if gender == required_gender {
                        return Some(dec);
                    }
                }
                // Try demutating after hyphen
                for &(mutated, original) in DEMUT_PAIRS {
                    if tail.starts_with(mutated) {
                        let candidate = format!("{}{}", original, &tail[mutated.len()..]);
                        if let Some(&(dec, gender)) = self.entries.get(candidate.as_str()) {
                            if gender == required_gender {
                                return Some(dec);
                            }
                        }
                    }
                }
            }
        }

        let chars: Vec<(usize, char)> = lemma.char_indices().collect();

        // Strategy 2: lenition detection
        for ci in 1..chars.len() {
            let (byte_pos, _) = chars[ci];
            let prefix = &lemma[..byte_pos];

            if prefix.len() < 3 {
                continue;
            }

            let tail = &lemma[byte_pos..];

            for &(mutated, original) in DEMUT_PAIRS {
                if tail.starts_with(mutated) {
                    let candidate = format!("{}{}", original, &tail[mutated.len()..]);
                    // Head must be >= 5 chars to avoid spurious short matches
                    if candidate.len() >= 5
                        && candidate != lemma
                        && !COMPOUND_HEAD_BLACKLIST.contains(&candidate.as_str())
                    {
                        if let Some(&(dec, gender)) = self.entries.get(candidate.as_str()) {
                            if gender == required_gender {
                                if best.is_none() || candidate.len() > best.unwrap().1 {
                                    best = Some((dec, candidate.len()));
                                }
                            }
                        }
                    }
                }
            }
        }

        if best.is_some() {
            return best.map(|(dec, _)| dec);
        }

        // Strategy 3: direct suffix match for long known heads (>= 7 chars).
        // Catches compounds without lenition like bun+tarraingt, bog+scrios.
        // High min length avoids false matches on short common suffixes.
        for ci in 1..chars.len() {
            let (byte_pos, _) = chars[ci];
            let prefix = &lemma[..byte_pos];
            let tail = &lemma[byte_pos..];

            if prefix.len() < 2 || tail.len() < 7 || tail == lemma {
                continue;
            }

            if let Some(&(dec, gender)) = self.entries.get(tail) {
                if gender == required_gender
                    && !COMPOUND_HEAD_BLACKLIST.contains(&tail)
                {
                    if best.is_none() || tail.len() > best.unwrap().1 {
                        best = Some((dec, tail.len()));
                    }
                }
            }
        }

        best.map(|(dec, _)| dec)
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
    // Téarma classless words where the guesser gets the wrong declension —
    // verified against BuNaMo XML and Kaikki genitives. These 16 entries
    // capture the entire accuracy gain that BuNaMo/Kaikki lookup would
    // provide on the classless Téarma subset.
    ("ab", (3, "aba")),
    ("ac", (4, "ac")),
    ("aip", (2, "aipe")),
    ("ar", (1, "air")),
    ("boilg", (2, "boilge")),
    ("scríobh", (3, "scríofa")),
    ("anas", (1, "anais")),
    ("bar", (1, "bair")),
    ("cat", (1, "cait")),
    ("col", (1, "coil")),
    ("fás", (1, "fáis")),
    ("gal", (1, "gail")),
    ("salm", (1, "sailm")),
    ("solas", (1, "solais")),
    ("teastas", (1, "teastais")),
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

/// Guess noun declension with compound decomposition.
///
/// Compound decomposition is tried **first**: Irish compounds always
/// take the declension of their final element (head word), so if we
/// can identify a known head word (including through lenition at the
/// join point), we trust it over morphological rules. The rules are
/// the fallback for non-compound words.
pub fn guess_declension_compound(lemma: &str, gender: Gender, db: &LemmaDb) -> Declension {
    // Exception lookups always take priority
    if let Some((dec, _)) = lookup_fully_irregular(lemma) {
        return Declension::from_i8(dec).unwrap_or(Declension::Irregular);
    }

    // Compound decomposition: if the word ends with a known lemma
    // (possibly lenited at the join), inherit its declension.
    // Only match when head is shorter than the full word (i.e. there's a prefix).
    if let Some(head_dec) = db.find_compound_head(lemma, gender) {
        return Declension::from_i8(head_dec).unwrap_or(Declension::Third);
    }

    // Fall back to morphological rules
    guess_declension(lemma, gender)
}

fn is_fifth_simple(_lemma: &str, _gender: Gender) -> bool {
    false
}

fn is_fourth_simple(lemma: &str, gender: Gender) -> bool {
    // Loanwords with consonant ending → 4th
    if in_list(lemma, POSSIBLE_LOANWORDS_GENITIVELESS) {
        if let Some(last) = lemma.chars().last() {
            if opers::CONSONANTS.contains(last.to_lowercase().next().unwrap_or(last)) {
                return true;
            }
        }
    }

    // Fem vowel-ending → 4th (97% of fem vowel-ending nouns in BuNaMo are 4th;
    // the few 5th-decl exceptions like caora, pearsa are already guessed wrong as 3rd)
    if gender == Gender::Fem {
        if let Some(last) = lemma.chars().last() {
            if opers::VOWELS.contains(last) {
                return true;
            }
        }
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
    if re_ends(lemma, &["eog", "óg", "lann", "ach"]) {
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
    // FAMILY → 5th (check before IRREGULAR_DECLENSION so deirfiúr/siúr get 5th)
    if in_list(lemma, FAMILY) {
        return Declension::Fifth;
    }

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

/// Build the full singular paradigm (nom/gen/voc/dat) for a noun.
///
/// Dispatches to the appropriate SingularInfo strategy per declension,
/// with overrides for FULLY_IRREGULAR and IRREGULAR_DECLENSION words.
pub fn singular_paradigm(lemma: &str, gender: Gender, declension: i8) -> Option<SingularInfo> {
    // IRREGULAR_DECLENSION — deeply irregular, only genitive is known
    for &(l, gen) in IRREGULAR_DECLENSION_BASE {
        if l == lemma {
            let si = SingularInfo {
                gender,
                nominative: vec![Form::new(lemma)],
                genitive: vec![Form::new(gen)],
                vocative: vec![Form::new(lemma)],
                dative: vec![Form::new(lemma)],
            };
            return Some(si);
        }
    }

    // FULLY_IRREGULAR — use standard strategy for voc/dat, override genitive
    if let Some((dec, gen)) = lookup_fully_irregular(lemma) {
        if dec == declension {
            let mut si = singular_paradigm_for_declension(lemma, gender, declension)
                .unwrap_or_else(|| singular_info::singular_info_o(lemma, gender));
            si.genitive = vec![Form::new(gen)];
            return Some(si);
        }
    }

    singular_paradigm_for_declension(lemma, gender, declension)
}

fn singular_paradigm_for_declension(lemma: &str, gender: Gender, declension: i8) -> Option<SingularInfo> {
    match declension {
        1 => Some(singular_paradigm_1st(lemma, gender)),
        2 => Some(singular_paradigm_2nd(lemma, gender)),
        3 => Some(singular_paradigm_3rd(lemma, gender)),
        4 => Some(singular_paradigm_4th(lemma, gender)),
        5 => Some(singular_paradigm_5th(lemma, gender)),
        _ => None,
    }
}

/// Generate the genitive form for a noun given lemma, gender, and declension.
pub fn generate_genitive(lemma: &str, gender: Gender, declension: i8) -> Option<String> {
    singular_paradigm(lemma, gender, declension)
        .and_then(|si| si.genitive.first().map(|f| f.value.clone()))
}

fn singular_paradigm_1st(lemma: &str, gender: Gender) -> SingularInfo {
    let target = IRREGULARLY_PALATALIZED_1ST.iter()
        .find(|(l, _)| *l == lemma)
        .map(|(_, t)| *t)
        .unwrap_or("");

    let with_iai = !in_list(lemma, WITHOUT_IAI);

    singular_info::singular_info_c(lemma, gender, target, with_iai)
}

fn singular_paradigm_2nd(lemma: &str, gender: Gender) -> SingularInfo {
    let ei_target = if in_list(lemma, POLYSYLLABIC_EI_2ND) {
        "ei"
    } else {
        ""
    };

    let use_mono_ei = !in_list(lemma, MONOSYLLABIC_I_2ND);

    if gender == Gender::Fem
        && (lemma.ends_with("ach") || lemma.ends_with("each"))
        && opers::polysyllabic(lemma)
    {
        return singular_info::singular_info_c(lemma, gender, ei_target, false);
    }

    singular_info::singular_info_e(lemma, gender, false, false, ei_target, use_mono_ei)
}

fn singular_paradigm_3rd(lemma: &str, gender: Gender) -> SingularInfo {
    let with_syncopated_ai = !in_list(lemma, UNSYNCOPATED_3RD);

    singular_info::singular_info_a(lemma, gender, false, "", with_syncopated_ai)
}

fn singular_paradigm_4th(lemma: &str, gender: Gender) -> SingularInfo {
    singular_info::singular_info_o(lemma, gender)
}

fn singular_paradigm_5th(lemma: &str, gender: Gender) -> SingularInfo {
    let last_char = lemma.chars().last().unwrap_or('a');

    if gender == Gender::Fem {
        if matches!(last_char, 'r' | 'l' | 'n') {
            return singular_paradigm_5th_fem_consonant(lemma, gender);
        }
        if opers::VOWELS.contains(last_char) {
            return singular_info::singular_info_n(lemma, gender);
        }
    }

    if gender == Gender::Masc && opers::VOWELS.contains(last_char) {
        return singular_info::singular_info_d(lemma, gender);
    }

    singular_info::singular_info_l(lemma, gender, "")
}

fn singular_paradigm_5th_fem_consonant(lemma: &str, gender: Gender) -> SingularInfo {
    let ax_word_endings = [
        "thir", "mhir", "eoir", "athair", "ochair", "bhair", "eorainn",
    ];
    let ax_substring_patterns = ["riai", "tiúi"];

    let has_ax_ending = ax_word_endings.iter().any(|e| lemma.ends_with(e));
    let has_ax_pattern = ax_substring_patterns.iter().any(|p| lemma.contains(p));

    if has_ax_ending || has_ax_pattern {
        let do_syncope = opers::polysyllabic(lemma);
        return singular_info::singular_info_ax(lemma, gender, do_syncope, "");
    }

    let broadening_endings = ["eoil", "coil", "ain", "ill"];

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
        return singular_info::singular_info_l(lemma, gender, "");
    }

    let do_syncope = opers::polysyllabic(lemma);
    singular_info::singular_info_ax(lemma, gender, do_syncope, "")
}

// ============================================================
// Plural paradigm generation — heuristic defaults per declension
// ============================================================

/// Generate a default plural paradigm from lemma, gender, and declension.
///
/// Mirrors the guesser's sub-classification within each declension so the
/// plural strategy is consistent with *why* the word has that declension.
/// BuNaMo-attested forms are preferable when available.
pub fn plural_paradigm(lemma: &str, gender: Gender, declension: i8) -> Option<PluralInfo> {
    match declension {
        1 => Some(plural_paradigm_1st(lemma)),
        2 => Some(plural_paradigm_2nd(lemma, gender)),
        3 => Some(plural_paradigm_3rd(lemma, gender)),
        4 => Some(plural_paradigm_4th(lemma, gender)),
        5 => None, // too irregular
        _ => None,
    }
}

/// 1st decl (masc, broad consonant): weak plural by slenderization (LgC).
/// Mirrors is_first_simple: gender == Masc && !is_slender.
fn plural_paradigm_1st(lemma: &str) -> PluralInfo {
    let target = IRREGULARLY_PALATALIZED_1ST.iter()
        .find(|(l, _)| *l == lemma)
        .map(|(_, t)| *t)
        .unwrap_or("");
    plural_info::plural_info_lgc(lemma, target)
}

/// 2nd decl plural, mirroring is_second_simple sub-conditions.
fn plural_paradigm_2nd(lemma: &str, _gender: Gender) -> PluralInfo {
    // -ach (polysyllabic) → strong -aí: bolgach → bolgaí
    // Mirrors the singular_paradigm_2nd polysyllabic -ach/-each branch
    if (lemma.ends_with("ach") || lemma.ends_with("each")) && opers::polysyllabic(lemma) {
        let base = if lemma.ends_with("each") {
            &lemma[..lemma.len() - "each".len()]
        } else {
            &lemma[..lemma.len() - "ach".len()]
        };
        return plural_info::plural_info_tr(&format!("{}aí", base));
    }
    // -eog/-óg → LgA (broaden + a): bróg → bróga, fuinneog → fuinneoga
    // Mirrors re_ends(lemma, &["eog", "óg"]) in is_second_simple
    if re_ends(lemma, &["eog", "óg"]) {
        return plural_info::plural_info_lga(lemma, "");
    }
    // -lann → LgA: clann → clanna
    if lemma.ends_with("lann") {
        return plural_info::plural_info_lga(lemma, "");
    }
    // Slender consonant → LgE (slenderize + e)
    // Mirrors is_slender check in is_second_simple
    if opers::is_slender(lemma) {
        return plural_info::plural_info_lge(lemma, "");
    }
    // Remaining broad consonant (shouldn't happen often for 2nd decl) → LgA
    plural_info::plural_info_lga(lemma, "")
}

/// 3rd decl plural, mirroring is_third_simple sub-conditions.
fn plural_paradigm_3rd(lemma: &str, gender: Gender) -> PluralInfo {
    // Masc agent nouns -éir/-eoir/-óir/-úir → strong -í: dochtúir → dochtúirí
    // Mirrors re_ends(lemma, &["éir", "eoir", "óir", "úir"]) in is_third_simple
    if gender == Gender::Masc && re_ends(lemma, &["éir", "eoir", "óir", "úir"]) {
        return plural_info::plural_info_tr(&format!("{}í", lemma));
    }
    // Fem -cht (abstract nouns) → strong -aí: beannacht → beannachtaí
    // Mirrors "cht" in is_third_simple's fem patterns
    if gender == Gender::Fem && lemma.ends_with("cht") {
        return plural_info::plural_info_tr(&format!("{}aí", lemma));
    }
    // Fem -irt → strong -í: cosaint → cosaintí
    if gender == Gender::Fem && lemma.ends_with("irt") {
        return plural_info::plural_info_tr(&format!("{}í", lemma));
    }
    // Default: slender → -í, broad → -aí
    if opers::is_slender(lemma) {
        plural_info::plural_info_tr(&format!("{}í", lemma))
    } else {
        plural_info::plural_info_tr(&format!("{}aí", lemma))
    }
}

/// 4th decl plural, mirroring is_fourth_simple sub-conditions.
fn plural_paradigm_4th(lemma: &str, _gender: Gender) -> PluralInfo {
    // Loanwords → strong -anna: bus → busanna
    // Mirrors the POSSIBLE_LOANWORDS_GENITIVELESS check in is_fourth_simple
    if in_list(lemma, POSSIBLE_LOANWORDS_GENITIVELESS) {
        return plural_info::plural_info_tr(&format!("{}anna", lemma));
    }
    // -ín → strong -í: cailín → cailíní
    // Mirrors "ín" in is_fourth_simple's masc re_ends
    if lemma.ends_with("ín") {
        return plural_info::plural_info_tr(&format!("{}í", lemma));
    }
    // -e → drop + -í: buille → buillí, coiste → coistí
    if lemma.ends_with('e') {
        return plural_info::plural_info_tr(&format!("{}í", &lemma[..lemma.len() - 1]));
    }
    // -ú → -uithe: scrúdú → scrúduithe
    if lemma.ends_with("ú") {
        return plural_info::plural_info_tr(
            &format!("{}uithe", &lemma[..lemma.len() - 'ú'.len_utf8()])
        );
    }
    // Other vowel endings → -nna
    plural_info::plural_info_tr(&format!("{}nna", lemma))
}

// ============================================================
// Full paradigm construction
// ============================================================

impl Noun {
    /// Build a full noun paradigm from lemma, gender, and declension class.
    ///
    /// Computes singular forms (nom/gen/voc/dat) via the SingularInfo
    /// strategy for the declension, and plural forms via heuristic defaults.
    /// For words with BuNaMo-attested paradigms, prefer loading from XML.
    pub fn from_lemma_with_declension(lemma: &str, gender: Gender, declension: i8) -> Self {
        let si = singular_paradigm(lemma, gender, declension);
        let pi = plural_paradigm(lemma, gender, declension);

        if let Some(si) = si {
            Self::create_from_info(&si, pi.as_ref(), declension)
        } else {
            let mut noun = Self::from_lemma(lemma, gender);
            noun.declension = declension;
            noun
        }
    }
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

    // ---- singular_paradigm tests ----

    #[test]
    fn test_singular_paradigm_1st_returns_all_cases() {
        let si = singular_paradigm("bád", Gender::Masc, 1).unwrap();
        assert_eq!(si.nominative[0].value, "bád");
        assert_eq!(si.genitive[0].value, "báid");
        assert_eq!(si.vocative[0].value, "báid"); // masc 1st: voc = slenderized
        assert_eq!(si.dative[0].value, "bád");
    }

    #[test]
    fn test_singular_paradigm_2nd_fem() {
        let si = singular_paradigm("bróg", Gender::Fem, 2).unwrap();
        assert_eq!(si.nominative[0].value, "bróg");
        assert_eq!(si.genitive[0].value, "bróige");
        assert_eq!(si.vocative[0].value, "bróg"); // fem: voc unchanged
    }

    #[test]
    fn test_singular_paradigm_4th_gen_eq_nom() {
        let si = singular_paradigm("bainne", Gender::Masc, 4).unwrap();
        assert_eq!(si.nominative[0].value, "bainne");
        assert_eq!(si.genitive[0].value, "bainne");
        assert_eq!(si.vocative[0].value, "bainne");
        assert_eq!(si.dative[0].value, "bainne");
    }

    #[test]
    fn test_singular_paradigm_fully_irregular_overrides_genitive() {
        // laoch is FULLY_IRREGULAR (1, "laoich") — genitive overridden,
        // but voc/dat should come from the standard 1st decl strategy
        let si = singular_paradigm("laoch", Gender::Masc, 1).unwrap();
        assert_eq!(si.genitive[0].value, "laoich");
        assert_eq!(si.nominative[0].value, "laoch");
    }

    #[test]
    fn test_singular_paradigm_irregular_declension() {
        let si = singular_paradigm("bean", Gender::Fem, 0).unwrap();
        assert_eq!(si.genitive[0].value, "mná");
        assert_eq!(si.nominative[0].value, "bean");
    }

    // ---- plural_paradigm tests ----

    #[test]
    fn test_plural_1st_weak_slenderize() {
        let pi = plural_paradigm("bád", Gender::Masc, 1).unwrap();
        assert_eq!(pi.nominative[0].value, "báid");
        assert_eq!(pi.genitive[0].value, "bád"); // weak: gen = broadened = lemma
    }

    #[test]
    fn test_plural_2nd_og_lga() {
        // -óg triggers LgA (broaden+a), matching is_second_simple's re_ends
        let pi = plural_paradigm("bróg", Gender::Fem, 2).unwrap();
        assert_eq!(pi.nominative[0].value, "bróga");
    }

    #[test]
    fn test_plural_2nd_eog_lga() {
        let pi = plural_paradigm("fuinneog", Gender::Fem, 2).unwrap();
        assert_eq!(pi.nominative[0].value, "fuinneoga");
    }

    #[test]
    fn test_plural_2nd_lann_lga() {
        let pi = plural_paradigm("clann", Gender::Fem, 2).unwrap();
        assert_eq!(pi.nominative[0].value, "clanna");
    }

    #[test]
    fn test_plural_2nd_polysyllabic_ach() {
        let pi = plural_paradigm("bolgach", Gender::Fem, 2).unwrap();
        assert_eq!(pi.nominative[0].value, "bolgaí"); // strong -aí
        assert_eq!(pi.genitive[0].value, "bolgaí"); // strong: gen = nom
    }

    #[test]
    fn test_plural_3rd_masc_agent() {
        // Masc -úir → strong -í, matching is_third_simple's agent noun pattern
        let pi = plural_paradigm("dochtúir", Gender::Masc, 3).unwrap();
        assert_eq!(pi.nominative[0].value, "dochtúirí");
    }

    #[test]
    fn test_plural_3rd_fem_cht() {
        // Fem -cht → strong -aí, matching is_third_simple's "cht" pattern
        let pi = plural_paradigm("beannacht", Gender::Fem, 3).unwrap();
        assert_eq!(pi.nominative[0].value, "beannachtaí");
    }

    #[test]
    fn test_plural_4th_e_ending() {
        let pi = plural_paradigm("buille", Gender::Masc, 4).unwrap();
        assert_eq!(pi.nominative[0].value, "buillí");
    }

    #[test]
    fn test_plural_4th_loanword() {
        // Loanwords → strong -anna, matching is_fourth_simple's loanword check
        let pi = plural_paradigm("bus", Gender::Masc, 4).unwrap();
        assert_eq!(pi.nominative[0].value, "busanna");
    }

    #[test]
    fn test_plural_4th_in_ending() {
        // -ín → strong -í, matching is_fourth_simple's "ín" pattern
        let pi = plural_paradigm("cailín", Gender::Masc, 4).unwrap();
        assert_eq!(pi.nominative[0].value, "cailíní");
    }

    #[test]
    fn test_plural_5th_returns_none() {
        assert!(plural_paradigm("athair", Gender::Masc, 5).is_none());
    }

    // ---- from_lemma_with_declension tests ----

    #[test]
    fn test_from_lemma_with_declension_fills_all_fields() {
        let noun = Noun::from_lemma_with_declension("bád", Gender::Masc, 1);
        assert_eq!(noun.declension, 1);
        assert_eq!(noun.sg_nom[0].value, "bád");
        assert_eq!(noun.sg_gen[0].value, "báid");
        assert_eq!(noun.sg_voc[0].value, "báid");
        assert_eq!(noun.sg_dat[0].value, "bád");
        assert!(!noun.pl_nom.is_empty()); // has plural
        assert_eq!(noun.pl_nom[0].value, "báid");
    }
}
