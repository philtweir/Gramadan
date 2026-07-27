use crate::features::{Form, Strength};
use crate::opers;

/// The plural forms of a noun (nominative, genitive, vocative) with strength.
#[derive(Debug, Clone)]
pub struct PluralInfo {
    pub strength: Strength,
    pub nominative: Vec<Form>,
    pub genitive: Vec<Form>,
    pub vocative: Vec<Form>,
}

impl PluralInfo {
    fn new(strength: Strength) -> Self {
        Self {
            strength,
            nominative: Vec::new(),
            genitive: Vec::new(),
            vocative: Vec::new(),
        }
    }
}

/// Weak plural by slenderization. Genitive = broadened base, vocative = broadened + "a".
pub fn plural_info_lgc(base: &str, slenderization_target: &str) -> PluralInfo {
    let mut info = PluralInfo::new(Strength::Weak);

    // Genitive: broaden
    let gen_form = opers::broaden(base);
    info.genitive.push(Form::new(&gen_form));

    // Vocative: broadened + "a"
    info.vocative.push(Form::new(format!("{}a", gen_form)));

    // Nominative: slenderize (with -ch → -gh)
    let nom_base = if base.ends_with("ch") {
        format!("{}gh", &base[..base.len() - 2])
    } else {
        base.to_string()
    };
    let nom_form = opers::slenderize_with_target(&nom_base, slenderization_target);
    info.nominative.push(Form::new(nom_form));

    info
}

/// Weak plural by slenderization + suffix "-e".
pub fn plural_info_lge(base: &str, slenderization_target: &str) -> PluralInfo {
    let mut info = PluralInfo::new(Strength::Weak);

    let form = format!(
        "{}e",
        opers::slenderize_with_target(base, slenderization_target)
    );
    info.nominative.push(Form::new(&form));
    info.genitive.push(Form::new(opers::broaden(base)));
    info.vocative.push(Form::new(&form));

    info
}

/// Weak plural by broadening + suffix "-a".
pub fn plural_info_lga(base: &str, broadening_target: &str) -> PluralInfo {
    let mut info = PluralInfo::new(Strength::Weak);

    let form = format!(
        "{}a",
        opers::broaden_with_target(base, broadening_target, false)
    );
    info.nominative.push(Form::new(&form));
    info.genitive.push(Form::new(opers::broaden(base)));
    info.vocative.push(Form::new(&form));

    info
}

/// Strong plural: all cases identical.
pub fn plural_info_tr(base: &str) -> PluralInfo {
    let mut info = PluralInfo::new(Strength::Strong);
    info.nominative.push(Form::new(base));
    info.genitive.push(Form::new(base));
    info.vocative.push(Form::new(base));
    info
}
