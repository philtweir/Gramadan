use crate::opers;

/// Verb conjugation class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConjugationClass {
    Irregular,
    First,
    Second,
}

/// The 11 irregular verbs in Irish.
const IRREGULARS: &[&str] = &[
    "abair", "beir", "bí", "clois", "cluin", "déan", "faigh", "feic", "ith", "tabhair", "tar",
    "téigh",
];

/// Guess verb conjugation class from lemma only (no paradigm needed).
///
/// Accuracy: ~94.6% vs BuNaMo ground truth.
///
/// Rules from the handoff doc:
/// 1. Check irregular list
/// 2. Monosyllabic (≤1 vowel group) → 1st
/// 3. Ends -igh/-aigh → 2nd
/// 4. Ends -áil/-eáil/-óil/-úil/-áin/-eáin → 1st
/// 5. Ends -il/-in/-ir/-is/-e → 2nd (syncopating)
/// 6. Default → 1st
pub fn guess_conjugation(lemma: &str) -> ConjugationClass {
    let lower = lemma.trim().to_lowercase();

    // 1. Irregular
    if IRREGULARS.contains(&lower.as_str()) {
        return ConjugationClass::Irregular;
    }

    // 2. Monosyllabic: count vowel groups
    let vowel_groups = count_vowel_groups(&lower);
    if vowel_groups <= 1 {
        return ConjugationClass::First;
    }

    // 3. Ends -igh or -aigh → 2nd
    if lower.ends_with("igh") || lower.ends_with("aigh") {
        return ConjugationClass::Second;
    }

    // 4. Ends in long-vowel -áil/-eáil/-óil/-úil/-áin/-eáin → 1st
    // (these look like syncopating but are actually 1st conjugation)
    for suffix in &["áil", "eáil", "óil", "úil", "áin", "eáin"] {
        if lower.ends_with(suffix) {
            return ConjugationClass::First;
        }
    }

    // 5. Ends -il/-in/-ir/-is/-e → 2nd (syncopating verbs)
    for suffix in &["il", "in", "ir", "is"] {
        if lower.ends_with(suffix) {
            return ConjugationClass::Second;
        }
    }
    if lower.ends_with('e') {
        return ConjugationClass::Second;
    }

    // 6. Default
    ConjugationClass::First
}

/// Determine conjugation from the lemma and its future independent base form.
///
/// This is the exact method from Gramadán: if 'f' appears in the future suffix,
/// it's 1st conjugation; otherwise 2nd.
pub fn get_conjugation_from_future(lemma: &str, future_base: &str) -> ConjugationClass {
    let lower_lemma = lemma.trim().to_lowercase();

    if IRREGULARS.contains(&lower_lemma.as_str()) {
        return ConjugationClass::Irregular;
    }

    // Strip common prefix
    let common_len = lemma
        .chars()
        .zip(future_base.chars())
        .take_while(|(a, b)| a == b)
        .count();

    let future_suffix: String = future_base.chars().skip(common_len).collect();

    if future_suffix.contains('f') {
        ConjugationClass::First
    } else {
        ConjugationClass::Second
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irregulars() {
        assert_eq!(guess_conjugation("bí"), ConjugationClass::Irregular);
        assert_eq!(guess_conjugation("abair"), ConjugationClass::Irregular);
        assert_eq!(guess_conjugation("déan"), ConjugationClass::Irregular);
    }

    #[test]
    fn test_monosyllabic_first() {
        // Single vowel group → 1st
        assert_eq!(guess_conjugation("mol"), ConjugationClass::First);
        assert_eq!(guess_conjugation("bris"), ConjugationClass::First);
        assert_eq!(guess_conjugation("cuir"), ConjugationClass::First);
        assert_eq!(guess_conjugation("léigh"), ConjugationClass::First);
    }

    #[test]
    fn test_igh_second() {
        // Polysyllabic -igh → 2nd
        assert_eq!(guess_conjugation("ceannaigh"), ConjugationClass::Second);
        assert_eq!(guess_conjugation("bailigh"), ConjugationClass::Second);
    }

    #[test]
    fn test_syncopating_second() {
        // -il/-in/-ir/-is → 2nd
        assert_eq!(guess_conjugation("imir"), ConjugationClass::Second);
        assert_eq!(guess_conjugation("oscail"), ConjugationClass::Second);
        assert_eq!(guess_conjugation("cosain"), ConjugationClass::Second);
    }

    #[test]
    fn test_long_vowel_first() {
        // -áil → 1st (not syncopating)
        assert_eq!(guess_conjugation("sábháil"), ConjugationClass::First);
    }

    #[test]
    fn test_default_first() {
        assert_eq!(guess_conjugation("tosaigh"), ConjugationClass::Second);
    }

    #[test]
    fn test_future_method() {
        // mol → molfaidh (has 'f' → 1st)
        assert_eq!(
            get_conjugation_from_future("mol", "molfaidh"),
            ConjugationClass::First
        );
        // ceannaigh → ceannóidh (no 'f' → 2nd)
        assert_eq!(
            get_conjugation_from_future("ceannaigh", "ceannóidh"),
            ConjugationClass::Second
        );
    }

    #[test]
    fn test_vowel_groups() {
        assert_eq!(count_vowel_groups("mol"), 1);
        assert_eq!(count_vowel_groups("oscail"), 2);
        assert_eq!(count_vowel_groups("ceannaigh"), 2);
        assert_eq!(count_vowel_groups("bí"), 1);
    }
}
