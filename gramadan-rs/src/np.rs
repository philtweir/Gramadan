// SPDX-License-Identifier: AGPL-3.0-or-later
//! Noun-phrase (definite / articled) forms for a noun: the Irish definite article
//! plus the initial mutation it triggers, per case/number/gender. A port of
//! Gramadan's `NP.cs` (the `NP(Noun head)` constructor). Produces `an bhróg`,
//! `na mbróg`, etc., reusing `opers::mutate` for the mutations themselves.
//!
//! STATUS: first cut. Covers nominative + genitive, singular + plural, with and
//! without article - the cells the Gréasán declension grid shows in "definite"
//! mode. The dative-with-article is dialectal (northern vs southern `sgDatArt`)
//! and is NOT ported yet; `is_definite` (already-definite heads take no article)
//! is likewise a TODO. Validate against BuNaMo's `nounPhrase/*.xml` articled forms
//! (e.g. `an fhadhb mhór` / `na bhfadhbanna móra`).

use crate::features::{Form, FormSg, Gender, Mutation};
use crate::noun::Noun;
use crate::opers;

/// The forms a noun takes as the head of a definite noun phrase. `*_art` fields
/// carry the article + mutation; the bare fields are the article-less NP forms.
#[derive(Debug, Clone, Default)]
pub struct NounPhrase {
    pub sg_nom: Vec<FormSg>,
    pub sg_nom_art: Vec<FormSg>,
    pub sg_gen: Vec<FormSg>,
    pub sg_gen_art: Vec<FormSg>,
    pub pl_nom: Vec<Form>,
    pub pl_nom_art: Vec<Form>,
    pub pl_gen: Vec<Form>,
    pub pl_gen_art: Vec<Form>,
}

impl NounPhrase {
    /// Build noun-phrase forms from a noun's paradigm. Mirrors `NP(Noun head)`.
    pub fn from_noun(head: &Noun) -> Self {
        let mut np = NounPhrase::default();
        let nil_if_immut = |m: Mutation| if head.is_immutable { Mutation::Nil } else { m };

        // singular nominative: an + (masc: t-prothesis / fem: lenition)
        for f in &head.sg_nom {
            np.sg_nom.push(FormSg::new(&f.value, f.gender));
            let m = nil_if_immut(if f.gender == Gender::Masc { Mutation::PrefT } else { Mutation::Len3 });
            np.sg_nom_art
                .push(FormSg::new(format!("an {}", opers::mutate(m, &f.value)), f.gender));
        }

        // singular genitive: bare lenites only for proper nouns; articled uses
        // "an" + lenition (masc) or "na" + h-prothesis (fem).
        for f in &head.sg_gen {
            let bare = nil_if_immut(if head.is_proper { Mutation::Len1 } else { Mutation::Nil });
            np.sg_gen.push(FormSg::new(opers::mutate(bare, &f.value), f.gender));
            let (m, article) = if f.gender == Gender::Masc {
                (Mutation::Len3, "an")
            } else {
                (Mutation::PrefH, "na")
            };
            let m = nil_if_immut(m);
            np.sg_gen_art
                .push(FormSg::new(format!("{} {}", article, opers::mutate(m, &f.value)), f.gender));
        }

        // plural nominative: na + h-prothesis
        for f in &head.pl_nom {
            np.pl_nom.push(Form::new(&f.value));
            let m = nil_if_immut(Mutation::PrefH);
            np.pl_nom_art.push(Form::new(format!("na {}", opers::mutate(m, &f.value))));
        }

        // plural genitive: bare lenites only for proper nouns; articled uses
        // "na" + eclipsis.
        for f in &head.pl_gen {
            let bare = nil_if_immut(if head.is_proper { Mutation::Len1 } else { Mutation::Nil });
            np.pl_gen.push(Form::new(opers::mutate(bare, &f.value)));
            let m = nil_if_immut(Mutation::Ecl1);
            np.pl_gen_art.push(Form::new(format!("na {}", opers::mutate(m, &f.value))));
        }

        np
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::Gender;
    use crate::noun::Noun;

    #[test]
    fn fear_masc_np() {
        let np = NounPhrase::from_noun(&Noun::from_lemma_with_declension("fear", Gender::Masc, 1));
        assert_eq!(np.sg_nom_art[0].value, "an fear");
        assert_eq!(np.sg_gen_art[0].value, "an fhir");
        assert_eq!(np.pl_nom_art[0].value, "na fir");
        assert_eq!(np.pl_gen_art[0].value, "na bhfear");
    }

    #[test]
    fn brog_fem_np() {
        let np = NounPhrase::from_noun(&Noun::from_lemma_with_declension("bróg", Gender::Fem, 2));
        assert_eq!(np.sg_nom_art[0].value, "an bhróg");
        assert_eq!(np.sg_gen_art[0].value, "na bróige");
        assert_eq!(np.pl_gen_art[0].value, "na mbróg");
    }
}
