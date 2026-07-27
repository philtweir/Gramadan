use crate::features::{Form, Gender};
use crate::opers;

/// The singular forms of a noun or adjective (nominative, genitive, vocative, dative).
#[derive(Debug, Clone)]
pub struct SingularInfo {
    pub gender: Gender,
    pub nominative: Vec<Form>,
    pub genitive: Vec<Form>,
    pub vocative: Vec<Form>,
    pub dative: Vec<Form>,
}

impl SingularInfo {
    fn new(gender: Gender) -> Self {
        Self {
            gender,
            nominative: Vec::new(),
            genitive: Vec::new(),
            vocative: Vec::new(),
            dative: Vec::new(),
        }
    }
}

/// Class O: all cases identical (no change). Used for 4th declension nouns where gen=nom.
pub fn singular_info_o(lemma: &str, gender: Gender) -> SingularInfo {
    let mut info = SingularInfo::new(gender);
    let f = || Form::new(lemma);
    info.nominative.push(f());
    info.genitive.push(f());
    info.vocative.push(f());
    info.dative.push(f());
    info
}

/// Class C: genitive/vocative formed by slenderization.
///
/// For 1st declension. If `with_iai` is true and the word ends in `ia+consonants`,
/// uses `iai` as the slenderization target (handles most 1st decl ia-words).
pub fn singular_info_c(
    lemma: &str,
    gender: Gender,
    slenderization_target: &str,
    with_iai: bool,
) -> SingularInfo {
    use regex::Regex;

    let mut info = SingularInfo::new(gender);
    info.nominative.push(Form::new(lemma));
    info.dative.push(Form::new(lemma));

    // Determine effective target
    let effective_target = if with_iai
        && !lemma.ends_with("iasc")
        && Regex::new(&format!("ia[{}]+$", opers::CONSONANTS))
            .unwrap()
            .is_match(lemma)
    {
        if slenderization_target.is_empty() {
            "iai"
        } else {
            slenderization_target
        }
    } else {
        slenderization_target
    };

    // Derive vocative/genitive base: -ch → -gh before slenderizing
    let form = if lemma.ends_with("ch") {
        format!("{}gh", &lemma[..lemma.len() - 2])
    } else {
        lemma.to_string()
    };
    let form = opers::slenderize_with_target(&form, effective_target);

    // Vocative: slenderized for masc, unchanged for fem
    if gender == Gender::Fem {
        info.vocative.push(Form::new(lemma));
    } else {
        info.vocative.push(Form::new(&form));
    }

    // Genitive: for fem, -igh → -í
    let gen_form = if gender == Gender::Fem && form.ends_with("igh") {
        format!("{}í", &form[..form.len() - "igh".len()])
    } else {
        form
    };
    info.genitive.push(Form::new(gen_form));
    info
}

/// Class L: genitive formed by broadening. Used for 5th declension.
pub fn singular_info_l(
    lemma: &str,
    gender: Gender,
    broadening_target: &str,
) -> SingularInfo {
    let mut info = SingularInfo::new(gender);
    info.nominative.push(Form::new(lemma));
    info.vocative.push(Form::new(lemma));
    info.dative.push(Form::new(lemma));

    let form = opers::broaden_with_target(lemma, broadening_target, false);
    info.genitive.push(Form::new(form));
    info
}

/// Class E: genitive formed by slenderization + suffix "-e".
///
/// For 2nd declension. Handles syncope, -ngt→-ngth, -ú→-aith.
pub fn singular_info_e(
    lemma: &str,
    gender: Gender,
    do_syncope: bool,
    double_dative: bool,
    slenderization_target: &str,
    with_monosyllabic_ei: bool,
) -> SingularInfo {
    use regex::Regex;

    let mut info = SingularInfo::new(gender);
    info.nominative.push(Form::new(lemma));
    info.vocative.push(Form::new(lemma));

    let use_ei = with_monosyllabic_ei && !opers::polysyllabic(lemma);

    // Derive dative/genitive base
    let mut form = lemma.to_string();
    if do_syncope {
        form = opers::syncope(&form);
    }
    form = if use_ei {
        opers::slenderize_with_ei(&form)
    } else {
        opers::slenderize_with_target(&form, slenderization_target)
    };

    // Dative
    if double_dative {
        info.dative.push(Form::new(lemma));
        info.dative.push(Form::new(&form));
    } else {
        info.dative.push(Form::new(lemma));
    }

    // Continue to genitive: special consonant cluster rules
    lazy_static_re!(RE_NGTH, r"([eiéí])ngt$");
    form = RE_NGTH.replace(&form, "${1}ngth").to_string();

    // -áú → -áith
    if form.ends_with("áú") {
        let base = &form[..form.len() - "áú".len()];
        form = format!("{}áith", base);
    } else if form.ends_with("iú") {
        let base = &form[..form.len() - "iú".len()];
        form = format!("{}ith", base);
    } else if form.ends_with('ú') {
        // Must check after áú/iú
        let base = &form[..form.len() - 'ú'.len_utf8()];
        form = format!("{}aith", base);
    }

    form.push('e');
    info.genitive.push(Form::new(form));
    info
}

/// Class A: genitive formed by broadening + suffix "-a".
///
/// For 3rd declension. Handles syncope, -rt→-rth, -nnt→-nn, -nt→-n.
pub fn singular_info_a(
    lemma: &str,
    gender: Gender,
    do_syncope: bool,
    broadening_target: &str,
    with_syncopated_ai: bool,
) -> SingularInfo {
    use regex::Regex;

    let mut info = SingularInfo::new(gender);
    info.nominative.push(Form::new(lemma));
    info.vocative.push(Form::new(lemma));
    info.dative.push(Form::new(lemma));

    // Auto-syncope for -ain/-ail/-air endings
    let effective_syncope = do_syncope
        || (with_syncopated_ai
            && Regex::new(r"ai[nlr]$").unwrap().is_match(lemma));

    let mut form = lemma.to_string();

    // Special consonant cluster rules before syncope
    lazy_static_re!(RE_RT, r"([eiéí])rt$");
    form = RE_RT.replace(&form, "${1}rth").to_string();

    lazy_static_re!(RE_NNT, r"([eiéí])nnt$");
    form = RE_NNT.replace(&form, "${1}nn").to_string();

    lazy_static_re!(RE_NT, r"([eiéí])nt$");
    form = RE_NT.replace(&form, "${1}n").to_string();

    if effective_syncope {
        form = opers::syncope(&form);
    }
    form = opers::broaden_with_target(&form, broadening_target, true);
    form.push('a');
    info.genitive.push(Form::new(form));
    info
}

/// Class D: genitive appends "-d" (broad vowel) or "-ad" (slender vowel).
/// Used for 5th declension masculine.
pub fn singular_info_d(lemma: &str, gender: Gender) -> SingularInfo {
    let mut info = SingularInfo::new(gender);
    info.nominative.push(Form::new(lemma));
    info.vocative.push(Form::new(lemma));
    info.dative.push(Form::new(lemma));

    let form = if let Some(last) = lemma.chars().last() {
        if opers::VOWELS_BROAD.contains(last) {
            format!("{}d", lemma)
        } else if opers::VOWELS_SLENDER.contains(last) {
            format!("{}ad", lemma)
        } else {
            lemma.to_string()
        }
    } else {
        lemma.to_string()
    };
    info.genitive.push(Form::new(form));
    info
}

/// Class N: genitive appends "-n" (broad vowel) or "-an" (slender vowel).
/// Used for 5th declension feminine.
pub fn singular_info_n(lemma: &str, gender: Gender) -> SingularInfo {
    let mut info = SingularInfo::new(gender);
    info.nominative.push(Form::new(lemma));
    info.vocative.push(Form::new(lemma));
    info.dative.push(Form::new(lemma));

    let form = if let Some(last) = lemma.chars().last() {
        if opers::VOWELS_BROAD.contains(last) {
            format!("{}n", lemma)
        } else if opers::VOWELS_SLENDER.contains(last) {
            format!("{}an", lemma)
        } else {
            lemma.to_string()
        }
    } else {
        lemma.to_string()
    };
    info.genitive.push(Form::new(form));
    info
}

/// Class EAX: genitive formed by slenderization + suffix "-each".
/// Used for 5th declension.
pub fn singular_info_eax(
    lemma: &str,
    gender: Gender,
    do_syncope: bool,
    slenderization_target: &str,
) -> SingularInfo {
    let mut info = SingularInfo::new(gender);
    info.nominative.push(Form::new(lemma));
    info.vocative.push(Form::new(lemma));
    info.dative.push(Form::new(lemma));

    let mut form = lemma.to_string();
    if do_syncope {
        form = opers::syncope(&form);
    }
    form = opers::slenderize_with_target(&form, slenderization_target);
    form.push_str("each");
    info.genitive.push(Form::new(form));
    info
}

/// Class AX: genitive formed by broadening + suffix "-ach".
/// Used for 5th declension.
pub fn singular_info_ax(
    lemma: &str,
    gender: Gender,
    do_syncope: bool,
    broadening_target: &str,
) -> SingularInfo {
    let mut info = SingularInfo::new(gender);
    info.nominative.push(Form::new(lemma));
    info.vocative.push(Form::new(lemma));
    info.dative.push(Form::new(lemma));

    let mut form = lemma.to_string();
    if do_syncope {
        form = opers::syncope(&form);
    }
    form = opers::broaden_with_target(&form, broadening_target, false);
    form.push_str("ach");
    info.genitive.push(Form::new(form));
    info
}

/// Helper macro for lazy static regex (local to this module).
macro_rules! lazy_static_re {
    ($name:ident, $pat:expr) => {
        static $name: once_cell::sync::Lazy<Regex> =
            once_cell::sync::Lazy::new(|| Regex::new($pat).unwrap());
    };
}
use lazy_static_re;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singular_info_o() {
        let info = singular_info_o("bus", Gender::Masc);
        assert_eq!(info.nominative[0].value, "bus");
        assert_eq!(info.genitive[0].value, "bus");
    }

    #[test]
    fn test_singular_info_c_first_decl() {
        // 1st declension: bád → báid
        let info = singular_info_c("bád", Gender::Masc, "", false);
        assert_eq!(info.genitive[0].value, "báid");
        assert_eq!(info.vocative[0].value, "báid");
    }

    #[test]
    fn test_singular_info_c_fem() {
        // Fem: bacach → bacaí (ch→gh, slenderize, then -igh→-í)
        let info = singular_info_c("bacach", Gender::Fem, "", false);
        assert_eq!(info.genitive[0].value, "bacaí");
        assert_eq!(info.vocative[0].value, "bacach"); // fem vocative unchanged
    }

    #[test]
    fn test_singular_info_e_second_decl() {
        // 2nd declension: bróg → bróige
        let info = singular_info_e("bróg", Gender::Fem, false, false, "", false);
        assert_eq!(info.genitive[0].value, "bróige");
    }

    #[test]
    fn test_singular_info_a_third_decl() {
        // 3rd declension: éan → éin... actually broadening+a
        // Better test: rud → ruda? No, rud is 3rd → ruda? Let's use grúpa
        // Actually for SingularInfoA: the genitive is broaden + a
        // ceacht → ceachta (already broad, just add -a)
        let info = singular_info_a("ocht", Gender::Fem, false, "", false);
        assert_eq!(info.genitive[0].value, "ochta");
    }

    #[test]
    fn test_singular_info_d() {
        // cara → carad
        let info = singular_info_d("cara", Gender::Masc);
        assert_eq!(info.genitive[0].value, "carad");
    }

    #[test]
    fn test_singular_info_n() {
        // pearsa → pearsan? Actually vowel check:
        // 'a' is broad → persann? No: persana... wait.
        // Let's use: caora → caoran (broad vowel ending)
        let info = singular_info_n("caora", Gender::Fem);
        assert_eq!(info.genitive[0].value, "caoran");
    }
}
