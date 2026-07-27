/// Initial mutation types for Irish consonants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutation {
    /// No mutation
    Nil,
    /// Full lenition (séimhiú)
    Len1,
    /// Lenition without d/t/s
    Len2,
    /// Len2 + s→ts
    Len3,
    /// Full eclipsis (urú)
    Ecl1,
    /// Eclipsis without vowels
    Ecl1x,
    /// Eclipsis without t/d/vowels
    Ecl2,
    /// Ecl2 + s→ts
    Ecl3,
    /// t-prefixation
    PrefT,
    /// h-prefixation
    PrefH,
    /// Len1 + d' before vowels/f
    Len1D,
    /// Len2 + d' before vowels/f
    Len2D,
    /// Len3 + d' before vowels/f
    Len3D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Masc,
    Fem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strength {
    Strong,
    Weak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Number {
    Sg,
    Pl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Case {
    Nom,
    Gen,
    Voc,
    Dat,
}

/// A grammatical form (a surface string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form {
    pub value: String,
}

impl Form {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// A singular form with associated gender.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSg {
    pub value: String,
    pub gender: Gender,
}

impl FormSg {
    pub fn new(value: impl Into<String>, gender: Gender) -> Self {
        Self {
            value: value.into(),
            gender,
        }
    }
}

/// A plural genitive form with associated strength.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormPlGen {
    pub value: String,
    pub strength: Strength,
}

impl FormPlGen {
    pub fn new(value: impl Into<String>, strength: Strength) -> Self {
        Self {
            value: value.into(),
            strength,
        }
    }
}
