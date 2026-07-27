use once_cell::sync::Lazy;
use regex::Regex;

use crate::features::Mutation;

// Character classes for Irish orthography
pub const CONSONANTS: &str = "bcdfghjklmnpqrstvwxz";
pub const VOWELS: &str = "aeiouáéíóú";
pub const VOWELS_BROAD: &str = "aouáóú";
pub const VOWELS_SLENDER: &str = "eiéí";

// Uppercase variants for regex patterns
const VOWELS_UPPER: &str = "AEIOUÁÉÍÓÚ";

fn is_vowel(c: char) -> bool {
    VOWELS.contains(c) || VOWELS_UPPER.contains(c)
}

fn is_consonant(c: char) -> bool {
    let lc = c.to_lowercase().next().unwrap_or(c);
    CONSONANTS.contains(lc)
}

fn is_broad_vowel(c: char) -> bool {
    VOWELS_BROAD.contains(c)
}

fn is_slender_vowel(c: char) -> bool {
    VOWELS_SLENDER.contains(c)
}

// ---- Lazy regex helpers ----

macro_rules! lazy_re {
    ($name:ident, $pat:expr) => {
        static $name: Lazy<Regex> = Lazy::new(|| Regex::new($pat).unwrap());
    };
}

// Predicates
lazy_re!(RE_ENDS_DENTAL, "[dntsDNTS]$");
lazy_re!(
    RE_IS_SLENDER,
    "[eiéí][^aeiouáéíóú]+$"
);
lazy_re!(
    RE_IS_SLENDER_I,
    "[ií][^aeiouáéíóú]+$"
);
lazy_re!(
    RE_ENDS_VOWEL,
    "[aeiouáéíóúAEIOUÁÉÍÓÚ]$"
);
lazy_re!(
    RE_STARTS_VOWEL,
    "^[aeiouáéíóúAEIOUÁÉÍÓÚ]"
);
lazy_re!(
    RE_STARTS_F_VOWEL,
    "^[fF][aeiouáéíóúAEIOUÁÉÍÓÚ]"
);
lazy_re!(RE_STARTS_BILABIAL, "^[bmpBMP]");
lazy_re!(
    RE_STARTS_VOWEL_FHX,
    "^[aeiouáéíóúAEIOUÁÉÍÓÚ]"
);
lazy_re!(RE_STARTS_FH_NOT_LR, "(?i)^fh[^lr]");

/// Whether the string ends in a dental consonant (d, n, t, s).
pub fn ends_dental(txt: &str) -> bool {
    RE_ENDS_DENTAL.is_match(txt)
}

/// Whether the string ends in a slender consonant cluster
/// (slender vowel followed by one or more consonants).
pub fn is_slender(txt: &str) -> bool {
    RE_IS_SLENDER.is_match(txt)
}

/// Whether slenderness is caused by 'i'/'í' specifically (not 'e'/'é').
pub fn is_slender_i(txt: &str) -> bool {
    RE_IS_SLENDER_I.is_match(txt)
}

/// Whether the string ends in a vowel.
pub fn ends_vowel(txt: &str) -> bool {
    RE_ENDS_VOWEL.is_match(txt)
}

/// Whether the string starts with a vowel.
pub fn starts_vowel(txt: &str) -> bool {
    RE_STARTS_VOWEL.is_match(txt)
}

/// Whether the string starts with F followed by a vowel.
pub fn starts_f_vowel(txt: &str) -> bool {
    RE_STARTS_F_VOWEL.is_match(txt)
}

/// Whether the string starts with b, m, or p.
pub fn starts_bilabial(txt: &str) -> bool {
    RE_STARTS_BILABIAL.is_match(txt)
}

/// Whether the string starts with a vowel or 'fh' (but not 'fhl'/'fhr').
pub fn starts_vowel_fhx(txt: &str) -> bool {
    RE_STARTS_VOWEL_FHX.is_match(txt) || RE_STARTS_FH_NOT_LR.is_match(txt)
}

/// Whether a word is polysyllabic.
pub fn polysyllabic(text: &str) -> bool {
    let cns = CONSONANTS;
    let vws = VOWELS;
    let pattern = if ends_vowel(text) {
        format!(
            "[{cns}]+[{vws}]+[{cns}]+[{vws}]*$"
        )
    } else {
        format!(
            "[{vws}][{cns}]+[{vws}]+[{cns}]+$"
        )
    };
    Regex::new(&pattern).unwrap().is_match(text)
}

/// Whether a word ends in a slender consonant or slender vowel.
pub fn is_slender_ending(text: &str) -> bool {
    if let Some(last) = text.chars().last() {
        if is_slender_vowel(last) {
            return true;
        }
    }
    is_slender(text)
}

// ---- Core morphological operations ----

/// Apply an initial mutation to Irish text.
pub fn mutate(mutation: Mutation, text: &str) -> String {
    match mutation {
        Mutation::Nil => text.to_string(),
        Mutation::Len1 | Mutation::Len1D => {
            let ret = lenite_1(text);
            if matches!(mutation, Mutation::Len1D) {
                prefix_d_prime(&ret)
            } else {
                ret
            }
        }
        Mutation::Len2 | Mutation::Len2D => {
            let ret = lenite_2(text);
            if matches!(mutation, Mutation::Len2D) {
                prefix_d_prime(&ret)
            } else {
                ret
            }
        }
        Mutation::Len3 | Mutation::Len3D => {
            let ret = lenite_3(text);
            if matches!(mutation, Mutation::Len3D) {
                prefix_d_prime(&ret)
            } else {
                ret
            }
        }
        Mutation::Ecl1 => eclipsis_1(text),
        Mutation::Ecl1x => eclipsis_1x(text),
        Mutation::Ecl2 => eclipsis_2(text),
        Mutation::Ecl3 => eclipsis_3(text),
        Mutation::PrefT => prefix_t(text),
        Mutation::PrefH => prefix_h(text),
    }
}

/// Get the first character and remaining string, handling multi-byte chars.
fn split_first_char(s: &str) -> Option<(char, &str)> {
    let mut chars = s.chars();
    let first = chars.next()?;
    Some((first, chars.as_str()))
}

/// Check if the word starts with an "exotic" consonant+J combination (like Djibouti).
fn starts_exotic_j(text: &str) -> bool {
    lazy_re!(RE, "^[pbmftdcgPBMFTDCG][jJ]");
    RE.is_match(text)
}

fn lenite_1(text: &str) -> String {
    if starts_exotic_j(text) {
        return text.to_string();
    }
    // Lenite p, b, m, f, t, d, c, g
    lazy_re!(RE_MAIN, "^([pbmftdcgPBMFTDCG])(.*)$");
    if let Some(caps) = RE_MAIN.captures(text) {
        return format!("{}h{}", &caps[1], &caps[2]);
    }
    // Lenite s before r, n, l, or vowel
    lazy_re!(RE_S, "^([sS])([rnlRNLaeiouáéíóúAEIOUÁÉÍÓÚ].*)$");
    if let Some(caps) = RE_S.captures(text) {
        return format!("{}h{}", &caps[1], &caps[2]);
    }
    text.to_string()
}

fn lenite_2(text: &str) -> String {
    if starts_exotic_j(text) {
        return text.to_string();
    }
    // Lenite p, b, m, f, c, g (NOT d, t, s)
    lazy_re!(RE, "^([pbmfcgPBMFCG])(.*)$");
    if let Some(caps) = RE.captures(text) {
        return format!("{}h{}", &caps[1], &caps[2]);
    }
    text.to_string()
}

fn lenite_3(text: &str) -> String {
    if starts_exotic_j(text) {
        return text.to_string();
    }
    // Same as lenite_2 but also s→ts
    lazy_re!(RE_MAIN, "^([pbmfcgPBMFCG])(.*)$");
    if let Some(caps) = RE_MAIN.captures(text) {
        return format!("{}h{}", &caps[1], &caps[2]);
    }
    lazy_re!(RE_S, "^([sS])([rnlRNLaeiouáéíóúAEIOUÁÉÍÓÚ].*)$");
    if let Some(caps) = RE_S.captures(text) {
        return format!("t{}{}", &caps[1], &caps[2]);
    }
    text.to_string()
}

fn prefix_d_prime(text: &str) -> String {
    lazy_re!(RE, "^([aeiouáéíóúAEIOUÁÉÍÓÚfF])(.*)$");
    if let Some(caps) = RE.captures(text) {
        format!("d'{}{}", &caps[1], &caps[2])
    } else {
        text.to_string()
    }
}

fn eclipsis_1(text: &str) -> String {
    if let Some((first, rest)) = split_first_char(text) {
        let (prefix, matched) = match first {
            'p' | 'P' => ("b", true),
            'b' | 'B' => ("m", true),
            'f' | 'F' => ("bh", true),
            'c' | 'C' => ("g", true),
            'g' | 'G' => ("n", true),
            't' | 'T' => ("d", true),
            'd' | 'D' => ("n", true),
            _ => ("", false),
        };
        if matched {
            return format!("{prefix}{first}{rest}");
        }
        // Vowels: lowercase gets n-, uppercase gets n (no hyphen)
        if is_vowel(first) {
            if first.is_lowercase() {
                return format!("n-{first}{rest}");
            } else {
                return format!("n{first}{rest}");
            }
        }
    }
    text.to_string()
}

fn eclipsis_1x(text: &str) -> String {
    // Same as eclipsis_1 but leaves vowels unchanged
    if let Some((first, rest)) = split_first_char(text) {
        let (prefix, matched) = match first {
            'p' | 'P' => ("b", true),
            'b' | 'B' => ("m", true),
            'f' | 'F' => ("bh", true),
            'c' | 'C' => ("g", true),
            'g' | 'G' => ("n", true),
            't' | 'T' => ("d", true),
            'd' | 'D' => ("n", true),
            _ => ("", false),
        };
        if matched {
            return format!("{prefix}{first}{rest}");
        }
    }
    text.to_string()
}

fn eclipsis_2(text: &str) -> String {
    // Eclipsis without t, d, or vowels
    if let Some((first, rest)) = split_first_char(text) {
        let (prefix, matched) = match first {
            'p' | 'P' => ("b", true),
            'b' | 'B' => ("m", true),
            'f' | 'F' => ("bh", true),
            'c' | 'C' => ("g", true),
            'g' | 'G' => ("n", true),
            _ => ("", false),
        };
        if matched {
            return format!("{prefix}{first}{rest}");
        }
    }
    text.to_string()
}

fn eclipsis_3(text: &str) -> String {
    // Eclipsis 2 + s→ts
    let result = eclipsis_2(text);
    if result != text {
        return result;
    }
    lazy_re!(RE_S, "^([sS])([rnlRNLaeiouáéíóúAEIOUÁÉÍÓÚ].*)$");
    if let Some(caps) = RE_S.captures(text) {
        return format!("t{}{}", &caps[1], &caps[2]);
    }
    text.to_string()
}

fn prefix_t(text: &str) -> String {
    if let Some((first, rest)) = split_first_char(text) {
        if is_vowel(first) {
            if first.is_lowercase() {
                return format!("t-{first}{rest}");
            } else {
                return format!("t{first}{rest}");
            }
        }
    }
    text.to_string()
}

fn prefix_h(text: &str) -> String {
    if let Some((first, rest)) = split_first_char(text) {
        if is_vowel(first) {
            return format!("h{first}{rest}");
        }
    }
    text.to_string()
}

/// Reverse mutations: strip lenition, eclipsis, prefixation markers.
pub fn demutate(text: &str) -> String {
    let mut result = text.to_string();

    // bhf → f
    lazy_re!(RE_BHF, "^[bB][hH]([fF].*)$");
    if let Some(caps) = RE_BHF.captures(&result) {
        result = caps[1].to_string();
    }

    // Consonant + h → consonant (lenition)
    lazy_re!(RE_LEN, "^([bcdfgmpstBCDFGMPST])[hH](.*)$");
    if let Some(caps) = RE_LEN.captures(&result) {
        result = format!("{}{}", &caps[1], &caps[2]);
    }

    // Eclipsis prefixes
    let eclipsis_pairs: &[(&str, &str)] = &[
        ("^[mM]([bB].*)$", "mb→b"),
        ("^[gG]([cC].*)$", "gc→c"),
        ("^[nN]([dD].*)$", "nd→d"),
        ("^[nN]([gG].*)$", "ng→g"),
        ("^[bB]([pP].*)$", "bp→p"),
        ("^[tT]([sS].*)$", "ts→s"),
        ("^[dD]([tT].*)$", "dt→t"),
    ];
    for (pattern, _) in eclipsis_pairs {
        let re = Regex::new(pattern).unwrap();
        if let Some(caps) = re.captures(&result) {
            result = caps[1].to_string();
            break;
        }
    }

    // d'fh → f
    lazy_re!(RE_DFH, "^[dD]'([fF])[hH](.*)$");
    if let Some(caps) = RE_DFH.captures(&result) {
        result = format!("{}{}", &caps[1], &caps[2]);
    }

    // d' before vowel
    lazy_re!(RE_D_VOWEL, "^[dD]'([aeiouáéíóúAEIOUÁÉÍÓÚ].*)$");
    if let Some(caps) = RE_D_VOWEL.captures(&result) {
        result = caps[1].to_string();
    }

    // h- before vowel
    lazy_re!(RE_H, "^[hH]([aeiouáéíóúAEIOUÁÉÍÓÚ].*)$");
    if let Some(caps) = RE_H.captures(&result) {
        result = caps[1].to_string();
    }

    // n- before vowel
    lazy_re!(RE_N, "^[nN]-([aeiouáéíóúAEIOUÁÉÍÓÚ].*)$");
    if let Some(caps) = RE_N.captures(&result) {
        result = caps[1].to_string();
    }

    // Strip t-/n- prothesis
    lazy_re!(RE_TN, "^[tn]-");
    result = RE_TN.replace(&result, "").to_string();

    result
}

/// Slenderize: change the last broad vowel cluster before final consonants
/// to end in a slender vowel.
///
pub fn slenderize(base: &str) -> String {
    slenderize_with_target(base, "")
}

/// Slenderize with a specific target vowel cluster replacement.
/// If target is empty or doesn't end in a slender vowel, falls back to regular slenderization.
pub fn slenderize_with_target(base: &str, target: &str) -> String {
    if !target.is_empty() {
        // Check target ends in slender vowel
        if let Some(last) = target.chars().last() {
            if is_slender_vowel(last) {
                // Irregular slenderization: replace the broad vowel cluster with target
                let pattern = format!(
                    "^(.*?)[{vws}]*[{broad}]([{cns}]+)$",
                    vws = VOWELS,
                    broad = VOWELS_BROAD,
                    cns = CONSONANTS
                );
                let re = Regex::new(&pattern).unwrap();
                if let Some(caps) = re.captures(base) {
                    let prefix = caps.get(1).map_or("", |m| m.as_str());
                    return format!("{}{}{}", prefix, target, &caps[2]);
                }
                return base.to_string();
            }
        }
        // Target doesn't end slender — fall through to regular
    }

    // Regular slenderization: try specific mappings first
    let sources = ["ea", "éa", "ia", "ío", "io", "iu", "ae"];
    let targets = ["i", "éi", "éi", "í", "i", "i", "aei"];

    // Include broad vowels in the infix to handle words like suíomh, claíomh
    let infix = format!("{}{}", CONSONANTS, VOWELS_BROAD);

    for (source, target) in sources.iter().zip(targets.iter()) {
        let pattern = format!(
            "^(.*[{infix}])?{source}([{cns}]+)$",
            infix = infix,
            source = regex::escape(source),
            cns = CONSONANTS
        );
        let re = Regex::new(&pattern).unwrap();
        if let Some(caps) = re.captures(base) {
            let prefix = caps.get(1).map_or("", |m| m.as_str());
            return format!("{}{}{}", prefix, target, &caps[2]);
        }
    }

    // Generic case: insert "i" after the last broad vowel
    let pattern = format!(
        "^(.*[{broad}])([{cns}]+)$",
        broad = VOWELS_BROAD,
        cns = CONSONANTS
    );
    let re = Regex::new(&pattern).unwrap();
    if let Some(caps) = re.captures(base) {
        let prefix = &caps[1];
        return format!("{}i{}", prefix, &caps[2]);
    }

    base.to_string()
}

/// Slenderize with the `with_ei` flag (use "ei" instead of "i" for the "ea" mapping
/// when the word is monosyllabic).
pub fn slenderize_with_ei(base: &str) -> String {
    // For monosyllabic words, ea→ei instead of ea→i
    let is_poly = polysyllabic(base);

    let infix = format!("{}{}", CONSONANTS, VOWELS_BROAD);

    if !is_poly {
        // Try "ea" → "ei" first
        let pattern = format!(
            "^(.*[{infix}])?ea([{cns}]+)$",
            infix = infix,
            cns = CONSONANTS
        );
        let re = Regex::new(&pattern).unwrap();
        if let Some(caps) = re.captures(base) {
            let prefix = caps.get(1).map_or("", |m| m.as_str());
            return format!("{}ei{}", prefix, &caps[2]);
        }
    }

    // Fall through to regular
    slenderize(base)
}

/// Broaden: change the last slender vowel cluster before final consonants
/// to end in a broad vowel.
///
pub fn broaden(base: &str) -> String {
    broaden_impl(base, false)
}

/// Broaden with the `io→ea` mapping enabled (used in 3rd declension -a suffixing).
pub fn broaden_with_io(base: &str) -> String {
    broaden_impl(base, true)
}

fn broaden_impl(base: &str, with_io: bool) -> String {
    let sources: &[&str] = if with_io {
        &["ói", "ei", "éi", "i", "aí", "í", "ui", "io"]
    } else {
        &["ói", "ei", "éi", "i", "aí", "í", "ui"]
    };
    let targets: &[&str] = if with_io {
        &["ó", "ea", "éa", "ea", "aío", "ío", "o", "ea"]
    } else {
        &["ó", "ea", "éa", "ea", "aío", "ío", "o"]
    };

    for (source, target) in sources.iter().zip(targets.iter()) {
        let pattern = format!(
            "^(.*[{cns}])?{source}([{cns}]+)$",
            cns = CONSONANTS,
            source = regex::escape(source),
        );
        let re = Regex::new(&pattern).unwrap();
        if let Some(caps) = re.captures(base) {
            let prefix = caps.get(1).map_or("", |m| m.as_str());
            return format!("{}{}{}", prefix, target, &caps[2]);
        }
    }

    // Generic: remove trailing "i" before consonants
    let pattern = format!("^(.*)i([{cns}]+)$", cns = CONSONANTS);
    let re = Regex::new(&pattern).unwrap();
    if let Some(caps) = re.captures(base) {
        return format!("{}{}", &caps[1], &caps[2]);
    }

    base.to_string()
}

/// Broaden with a specific target vowel cluster.
/// If target doesn't end in a broad vowel, falls back to regular broadening.
pub fn broaden_with_target(base: &str, target: &str, with_io: bool) -> String {
    if !target.is_empty() {
        if let Some(last) = target.chars().last() {
            if is_broad_vowel(last) {
                let pattern = format!(
                    "^(.*?)[{vws}]*[{slender}]([{cns}]+)$",
                    vws = VOWELS,
                    slender = VOWELS_SLENDER,
                    cns = CONSONANTS
                );
                let re = Regex::new(&pattern).unwrap();
                if let Some(caps) = re.captures(base) {
                    return format!("{}{}{}", &caps[1], target, &caps[2]);
                }
                return base.to_string();
            }
        }
        // Target doesn't end broad — fall through
    }
    broaden_impl(base, with_io)
}

/// Devoice: if the final consonant cluster has a voicing mismatch, devoice.
/// Currently only handles sd → st.
pub fn devoice(base: &str) -> String {
    if base.ends_with("sd") {
        let mut result = base[..base.len() - 2].to_string();
        result.push_str("st");
        return result;
    }
    base.to_string()
}

/// Remove duplicated final consonants.
pub fn unduplicate(base: &str) -> String {
    let chars: Vec<char> = base.chars().collect();
    if chars.len() >= 2 {
        let last = chars[chars.len() - 1];
        let penult = chars[chars.len() - 2];
        if is_consonant(last) && last == penult {
            return chars[..chars.len() - 1].iter().collect();
        }
    }
    base.to_string()
}

/// Syncope: remove the final vowel cluster between consonant groups,
/// then unduplicate and devoice.
///
/// Performs syncope: removes the final vowel cluster between consonant groups,
pub fn syncope(base: &str) -> String {
    let pattern = format!(
        "^(.*[{cns}])?[{vws}]+([{cns}]+)$",
        cns = CONSONANTS,
        vws = VOWELS
    );
    let re = Regex::new(&pattern).unwrap();

    if let Some(caps) = re.captures(base) {
        let prefix = caps.get(1).map_or("", |m| m.as_str());
        let suffix = &caps[2];
        let combined = format!("{}{}", prefix, suffix);
        let mut result = devoice(&unduplicate(&combined));

        // Handle ln/dl/nl/dn/nd clusters
        if ["ln", "dl", "nl", "dn", "nd"]
            .iter()
            .any(|pair| result.contains(pair))
        {
            // Exceptions
            if result == "codladh" || result == "caibidl" {
                return result;
            }
            // Find the prefix that is unchanged between base and result
            let prefix_len = base
                .chars()
                .zip(result.chars())
                .take_while(|(b, r)| b == r)
                .count();
            let safe_start = if prefix_len >= 2 { prefix_len - 2 } else { 0 };
            let prefix_part: String = result.chars().take(safe_start).collect();
            let suffix_part: String = result.chars().skip(safe_start).collect();

            // Replace clusters, but only if not preceded by l/n/d
            lazy_re!(RE_LL, "([^lnd])(ln|dl|nl)");
            lazy_re!(RE_NN, "([^lnd])(dn|nd)");
            let s = RE_LL.replace_all(&suffix_part, "${1}ll").to_string();
            let s = RE_NN.replace_all(&s, "${1}nn").to_string();
            result = format!("{}{}", prefix_part, s);
        }

        return result;
    }

    base.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutate_lenition() {
        assert_eq!(mutate(Mutation::Len1, "bád"), "bhád");
        assert_eq!(mutate(Mutation::Len1, "cat"), "chat");
        assert_eq!(mutate(Mutation::Len1, "fear"), "fhear");
        assert_eq!(mutate(Mutation::Len1, "sean"), "shean");
        assert_eq!(mutate(Mutation::Len1, "leabhar"), "leabhar"); // l not lenitable
    }

    #[test]
    fn test_mutate_eclipsis() {
        assert_eq!(mutate(Mutation::Ecl1, "bád"), "mbád");
        assert_eq!(mutate(Mutation::Ecl1, "cat"), "gcat");
        assert_eq!(mutate(Mutation::Ecl1, "fear"), "bhfear");
        assert_eq!(mutate(Mutation::Ecl1, "teach"), "dteach");
        assert_eq!(mutate(Mutation::Ecl1, "athair"), "n-athair");
        assert_eq!(mutate(Mutation::Ecl1, "Éire"), "nÉire");
    }

    #[test]
    fn test_mutate_prefix_t() {
        assert_eq!(mutate(Mutation::PrefT, "athair"), "t-athair");
        assert_eq!(mutate(Mutation::PrefT, "Éire"), "tÉire");
        assert_eq!(mutate(Mutation::PrefT, "bean"), "bean"); // consonant, no change
    }

    #[test]
    fn test_demutate() {
        assert_eq!(demutate("bhád"), "bád");
        assert_eq!(demutate("gcat"), "cat");
        assert_eq!(demutate("n-athair"), "athair");
        assert_eq!(demutate("t-uisce"), "uisce");
    }

    #[test]
    fn test_slenderize() {
        assert_eq!(slenderize("bád"), "báid");
        assert_eq!(slenderize("fear"), "fir");
        // beart with default slenderize: ea→i (without ei flag)
        assert_eq!(slenderize("beart"), "birt");
        // With ei flag (monosyllabic): ea→ei
        assert_eq!(slenderize_with_ei("beart"), "beirt");
    }

    #[test]
    fn test_broaden() {
        assert_eq!(broaden("fir"), "fear");
        assert_eq!(broaden("báid"), "bád");
    }

    #[test]
    fn test_syncope() {
        assert_eq!(syncope("íseal"), "ísl");
    }

    #[test]
    fn test_is_slender() {
        // 'fear' ends in 'ear' — 'ea' is broad, so NOT slender
        assert!(!is_slender("fear"));
        assert!(is_slender("fir"));
        assert!(is_slender("báid"));
    }

    #[test]
    fn test_predicates() {
        assert!(ends_vowel("cailé"));
        assert!(!ends_vowel("cat"));
        assert!(starts_vowel("athair"));
        assert!(!starts_vowel("fear"));
        assert!(ends_dental("cat"));
        // 'd' IS a dental
        assert!(ends_dental("bád"));
        assert!(ends_dental("bán"));   // n is dental
        assert!(!ends_dental("bám"));  // m is not dental
    }
}
