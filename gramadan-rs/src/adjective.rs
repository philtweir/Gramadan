use crate::noun::Declension;
use crate::opers;

const VOWEL_ENDINGS: &[&str] = &["a", "e", "í", "o", "é", "ó", "á"];

pub fn guess_declension(lemma: &str) -> Declension {
    if lemma.ends_with("úil") {
        return Declension::Second;
    }

    if VOWEL_ENDINGS.iter().any(|v| lemma.ends_with(v)) {
        return Declension::Third;
    }

    Declension::First
}

#[derive(Debug, Clone)]
pub struct AdjectiveForms {
    pub sg_nom: String,
    pub sg_gen_masc: String,
    pub sg_gen_fem: String,
    pub sg_voc_masc: String,
    pub sg_voc_fem: String,
    pub pl_nom: String,
    pub graded: String,
}

const IRREGULAR_GRADED: &[(&str, &str)] = &[
    ("beag", "lú"),
    ("breá", "breátha"),
    ("ceanntréan", "ceanntreise"),
    ("fada", "faide"),
    ("furasta", "fusa"),
    ("gearr", "giorra"),
    ("iomdha", "lia"),
    ("maith", "fearr"),
    ("mór", "mó"),
    ("nua", "nuaí"),
    ("olc", "measa"),
    ("te", "teo"),
    ("tréan", "treise"),
    ("ábhalmhór", "ábhalmhó"),
    ("ollmhór", "ollmhó"),
];

// Dec-1 adjectives where genMasc == lemma (no slenderization),
// despite having a broad final vowel. Includes monosyllabic exceptions
// and words with -ocht/-cht endings.
const NO_CHANGE_MASC_GEN: &[&str] = &[
    "amh", "anghrách", "beacht", "breoch", "bách",
    "cairdiach", "caoch", "ceoch", "ciar", "corr", "cúng",
    "dealbh", "docht", "díleách", "díomách", "dúilmhear",
    "fann", "fionn",
    "gann", "gearr", "geamhchaoch", "gnách", "grách",
    "leamh", "leasc",
    "lomnocht", "mear", "moch", "nocht",
    "reoch", "searbh",
    "seang", "seasc", "spleách", "síorghnách",
    "taoidmhear", "tarnocht", "teann", "treoch", "trom",
    "tiubh", "tur",
    "tírghrách", "urghnách", "íon",
];

// Roots where ea→ei slenderization applies even in polysyllabic compounds.
// Each entry is (root, slenderized_root) — matches both plain and lenited forms.
const SLENDERIZE_EI_ROOTS: &[(&str, &str)] = &[
    ("dhearbh", "dheirbh"),
    ("dhearg", "dheirg"),
    ("dearbh", "deirbh"),
    ("dearg", "deirg"),
    ("dheas", "dheis"),
    ("deas", "deis"),
    ("tréan", "tréin"),
];

// Dec-1 adjectives with irregular genFem (syncope or special forms).
// These are used for genFem, plNom (often same), and graded (unless irregular).
// Compound adjectives ending in these roots inherit the transformation.
const SYNCOPATING_ADJ: &[(&str, &str)] = &[
    ("achomair", "achoimre"),
    ("anacair", "anacra"),
    ("baineann", "baininne"),
    ("beacht", "beaichte"),
    ("breac", "brice"),
    ("ceart", "cirte"),
    ("choiteann", "choitinne"),
    ("coiteann", "coitinne"),
    ("comair", "coimre"),
    ("cóir", "córa"),
    ("cúileann", "cúilinne"),
    ("daibhir", "daibhre"),
    ("daingean", "daingne"),
    ("deacair", "deacra"),
    ("dealbh", "deilbhe"),
    ("deimhin", "deimhne"),
    ("doilbhir", "doilbhre"),
    ("doiligh", "doilí"),
    ("domhain", "doimhne"),
    ("droimeann", "droiminne"),
    ("dílis", "dílse"),
    ("dúilmhear", "dúilmhire"),
    ("fireann", "firinne"),
    ("folamh", "foilmhe"),
    ("follas", "foilse"),
    ("geallmhar", "geallmhairme"),
    ("geal", "gile"),
    ("gearr", "giorra"),
    ("leasc", "leisce"),
    ("leamh", "leimhe"),
    ("leathan", "leithne"),
    ("láidir", "láidre"),
    ("mhaith", "mhaithe"),
    ("mhear", "mhire"),
    ("otair", "otra"),
    ("ramhar", "raimhre"),
    ("righin", "righne"),
    ("saibhir", "saibhre"),
    ("seang", "seinge"),
    ("seasc", "seisce"),
    ("searbh", "seirbhe"),
    ("seicear", "seicire"),
    ("socair", "socra"),
    ("soilbhir", "soilbhre"),
    ("taoidmhear", "taoidmhire"),
    ("teann", "teinne"),
    ("uasal", "uaisle"),
    ("uiríseal", "uirísle"),
    ("ábhal", "áibhle"),
    ("álainn", "áille"),
    ("íseal", "ísle"),
];

// Subset of SYNCOPATING_ADJ where plNom also uses the syncopated form.
// Words NOT here use regular plNom (lemma + -a/-e).
const SYNCOPATING_ADJ_PLNOM: &[(&str, &str)] = &[
    ("achomair", "achoimre"),
    ("anacair", "anacra"),
    ("comair", "coimre"),
    ("cóir", "córa"),
    ("daibhir", "daibhre"),
    ("daingean", "daingne"),
    ("deacair", "deacra"),
    ("deimhin", "deimhne"),
    ("doilbhir", "doilbhre"),
    ("doiligh", "doilí"),
    ("domhain", "doimhne"),
    ("dílis", "dílse"),
    ("folamh", "folmha"),
    ("láidir", "láidre"),
    ("otair", "otra"),
    ("righin", "righne"),
    ("saibhir", "saibhre"),
    ("sleamhain", "sleamhna"),
    ("socair", "socra"),
    ("soilbhir", "soilbhre"),
    ("uasal", "uaisle"),
    ("uiríseal", "uirísle"),
    ("álainn", "áille"),
    ("íseal", "ísle"),
];

// Roots where plNom uses syncope (broaden + -a instead of regular append).
// These are polysyllabic words ending in -al, -ar, -air where the internal
// vowel drops and -a is appended with broadening.
const SYNCOPATING_PLNOM: &[(&str, &str)] = &[
    ("bodhar", "bodhra"),
    ("diamhair", "diamhra"),
    ("easumhal", "easumhla"),
    ("odhar", "odhra"),
    ("ramhar", "ramhra"),
    ("umhal", "umhla"),
];

const AMBIGUOUS_ADJECTIVES: &[&str] = &[
    "ceanntréan",
    "fíormhaith",
    "tréan",
];

pub fn generate_forms(lemma: &str, declension: Declension) -> Option<AdjectiveForms> {
    if AMBIGUOUS_ADJECTIVES.iter().any(|&w| w == lemma) {
        return None;
    }
    Some(match declension {
        Declension::First => generate_first(lemma),
        Declension::Second => generate_second(lemma),
        Declension::Third => generate_third(lemma),
        _ => generate_third(lemma),
    })
}

fn generate_first(lemma: &str) -> AdjectiveForms {
    let sg_gen_masc = gen_masc_first(lemma);
    let sg_gen_fem = gen_fem_first(lemma);
    let sg_voc_masc = sg_gen_masc.clone();
    let sg_voc_fem = lemma.to_string();
    let pl_nom = pl_nom_first(lemma);
    let graded = lookup_irregular_graded(lemma)
        .unwrap_or_else(|| sg_gen_fem.clone());

    AdjectiveForms {
        sg_nom: lemma.to_string(),
        sg_gen_masc,
        sg_gen_fem,
        sg_voc_masc,
        sg_voc_fem,
        pl_nom,
        graded,
    }
}

fn gen_masc_first(lemma: &str) -> String {
    // Already slender → no change
    if opers::is_slender(lemma) {
        return lemma.to_string();
    }

    // -íoch/-éach/-uach/-ách endings → no change
    if lemma.ends_with("íoch") || lemma.ends_with("éach")
        || lemma.ends_with("uach") || lemma.ends_with("ách")
    {
        return lemma.to_string();
    }

    // Known broad-ending no-change words
    if NO_CHANGE_MASC_GEN.contains(&lemma) {
        return lemma.to_string();
    }

    // -ach → -aigh, -each → -igh (via -ch → -gh then slenderize)
    if lemma.ends_with("ch") {
        let base = format!("{}gh", &lemma[..lemma.len() - 2]);
        return opers::slenderize(&base);
    }

    // Compound adjectives ending in roots that use ea→ei slenderization
    for &(root, slenderized) in SLENDERIZE_EI_ROOTS {
        if lemma.ends_with(root) {
            let prefix = &lemma[..lemma.len() - root.len()];
            return format!("{}{}", prefix, slenderized);
        }
    }

    // General: slenderize
    opers::slenderize(lemma)
}

fn gen_fem_first(lemma: &str) -> String {
    // -éach → replace -éach with -éiche
    if lemma.ends_with("éach") {
        return format!("{}éiche", &lemma[..lemma.len() - "éach".len()]);
    }
    // -uach → replace -uach with -uaiche
    if lemma.ends_with("uach") {
        return format!("{}uaiche", &lemma[..lemma.len() - "uach".len()]);
    }
    // -ách → replace -ách with -áiche
    if lemma.ends_with("ách") {
        return format!("{}áiche", &lemma[..lemma.len() - "ách".len()]);
    }

    // -each words: -each → -í
    if lemma.ends_with("each") {
        return format!("{}í", &lemma[..lemma.len() - "each".len()]);
    }
    // -iach words: -iach → -iaiche
    if lemma.ends_with("iach") {
        return format!("{}iaiche", &lemma[..lemma.len() - "iach".len()]);
    }
    // -ach words: -ach → -aí
    if lemma.ends_with("ach") {
        return format!("{}aí", &lemma[..lemma.len() - "ach".len()]);
    }

    // -íoch → -íche
    if lemma.ends_with("íoch") {
        return format!("{}che", &lemma[..lemma.len() - "och".len()]);
    }

    // Known syncopating adjectives (and compounds ending in them)
    if let Some(form) = lookup_syncopating(lemma) {
        return form;
    }

    // Already slender: just append -e
    if opers::is_slender(lemma) {
        return format!("{}e", lemma);
    }

    // Compound adjectives with ea→ei roots
    for &(root, slenderized) in SLENDERIZE_EI_ROOTS {
        if lemma.ends_with(root) {
            let prefix = &lemma[..lemma.len() - root.len()];
            return format!("{}{}e", prefix, slenderized);
        }
    }

    // General: slenderize + -e (without syncope)
    let slenderized = opers::slenderize(lemma);
    format!("{}e", slenderized)
}

fn lookup_syncopating(lemma: &str) -> Option<String> {
    lookup_in_table(lemma, SYNCOPATING_ADJ)
}

fn lookup_syncopating_plnom(lemma: &str) -> Option<String> {
    lookup_in_table(lemma, SYNCOPATING_ADJ_PLNOM)
}

fn lookup_in_table(lemma: &str, table: &[(&str, &str)]) -> Option<String> {
    // Direct match
    if let Some((_, form)) = table.iter().find(|(l, _)| *l == lemma) {
        return Some(form.to_string());
    }
    // Compound match: if lemma ends with a root, apply the
    // same transformation to the compound
    for &(root, form) in table {
        if lemma.len() > root.len() && lemma.ends_with(root) {
            let prefix = &lemma[..lemma.len() - root.len()];
            return Some(format!("{}{}", prefix, form));
        }
    }
    None
}

fn pl_nom_first(lemma: &str) -> String {
    // -ach/-each → append -a
    if lemma.ends_with("ach") || lemma.ends_with("each") {
        return format!("{}a", lemma);
    }

    // -íoch/-éach/-uach/-ách → append -a
    if lemma.ends_with("íoch") || lemma.ends_with("éach")
        || lemma.ends_with("uach") || lemma.ends_with("ách")
    {
        return format!("{}a", lemma);
    }

    // -eann words: append -a (not syncopated)
    if lemma.ends_with("eann") {
        return format!("{}a", lemma);
    }

    // Known syncopating adjectives where plNom == genFem
    if let Some(form) = lookup_syncopating_plnom(lemma) {
        return form;
    }

    // Known syncopating plNom (broaden + -a pattern)
    if let Some(form) = lookup_in_table(lemma, SYNCOPATING_PLNOM) {
        return form;
    }

    // Slender → append -e
    if opers::is_slender(lemma) {
        return format!("{}e", lemma);
    }

    // Broad → append -a
    format!("{}a", lemma)
}

fn generate_second(lemma: &str) -> AdjectiveForms {
    let broadened = opers::broaden(lemma);
    let gen_fem = format!("{}a", broadened);

    let graded = lookup_irregular_graded(lemma)
        .unwrap_or_else(|| gen_fem.clone());

    AdjectiveForms {
        sg_nom: lemma.to_string(),
        sg_gen_masc: lemma.to_string(),
        sg_gen_fem: gen_fem.clone(),
        sg_voc_masc: lemma.to_string(),
        sg_voc_fem: lemma.to_string(),
        pl_nom: gen_fem,
        graded,
    }
}

const IRREGULAR_PLNOM_DEC3: &[(&str, &str)] = &[
    ("breá", "breátha"),
    ("te", "teo"),
];

fn generate_third(lemma: &str) -> AdjectiveForms {
    let graded = lookup_irregular_graded(lemma)
        .unwrap_or_else(|| lemma.to_string());

    let pl_nom = IRREGULAR_PLNOM_DEC3.iter()
        .find(|(l, _)| *l == lemma)
        .map(|(_, p)| p.to_string())
        .unwrap_or_else(|| lemma.to_string());

    let sg_gen_fem = if lemma == "breá" { "breátha".to_string() }
        else if lemma == "nua" { "nuaí".to_string() }
        else { lemma.to_string() };

    AdjectiveForms {
        sg_nom: lemma.to_string(),
        sg_gen_masc: lemma.to_string(),
        sg_gen_fem,
        sg_voc_masc: lemma.to_string(),
        sg_voc_fem: lemma.to_string(),
        pl_nom,
        graded,
    }
}

fn lookup_irregular_graded(lemma: &str) -> Option<String> {
    IRREGULAR_GRADED.iter()
        .find(|(l, _)| *l == lemma)
        .map(|(_, g)| g.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_first_decl() {
        assert_eq!(guess_declension("mór"), Declension::First);
        assert_eq!(guess_declension("bán"), Declension::First);
        assert_eq!(guess_declension("gorm"), Declension::First);
        assert_eq!(guess_declension("beag"), Declension::First);
        assert_eq!(guess_declension("cúthail"), Declension::First);
    }

    #[test]
    fn test_guess_second_decl() {
        assert_eq!(guess_declension("dathúil"), Declension::Second);
        assert_eq!(guess_declension("misniúil"), Declension::Second);
        assert_eq!(guess_declension("cumhachtúil"), Declension::Second);
    }

    #[test]
    fn test_guess_third_decl() {
        assert_eq!(guess_declension("dána"), Declension::Third);
        assert_eq!(guess_declension("hálainn"), Declension::First);
        assert_eq!(guess_declension("breá"), Declension::Third);
        assert_eq!(guess_declension("cruthaithe"), Declension::Third);
    }

    #[test]
    fn test_gen_masc_first_ach() {
        let f = generate_forms("deireanach", Declension::First).unwrap();
        assert_eq!(f.sg_gen_masc, "deireanaigh");
    }

    #[test]
    fn test_gen_fem_first_ach() {
        let f = generate_forms("deireanach", Declension::First).unwrap();
        assert_eq!(f.sg_gen_fem, "deireanaí");
    }

    #[test]
    fn test_gen_second_decl() {
        let f = generate_forms("dathúil", Declension::Second).unwrap();
        assert_eq!(f.sg_gen_masc, "dathúil");
        assert_eq!(f.sg_gen_fem, "dathúla");
        assert_eq!(f.pl_nom, "dathúla");
    }

    #[test]
    fn test_gen_third_decl() {
        let f = generate_forms("dána", Declension::Third).unwrap();
        assert_eq!(f.sg_gen_masc, "dána");
        assert_eq!(f.sg_gen_fem, "dána");
    }

    #[test]
    fn test_irregular_graded() {
        let f = generate_forms("maith", Declension::First).unwrap();
        assert_eq!(f.graded, "fearr");

        let f = generate_forms("mór", Declension::First).unwrap();
        assert_eq!(f.graded, "mó");

        let f = generate_forms("breá", Declension::Third).unwrap();
        assert_eq!(f.graded, "breátha");
    }

    #[test]
    fn test_syncope_gen_fem() {
        let f = generate_forms("daibhir", Declension::First).unwrap();
        assert_eq!(f.sg_gen_fem, "daibhre");

        let f = generate_forms("láidir", Declension::First).unwrap();
        assert_eq!(f.sg_gen_fem, "láidre");
    }
}
