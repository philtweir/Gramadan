from dataclasses import dataclass
from enum import Enum, auto

# Enumerations for various grammatical features:


class Mutation:
    Nil = auto()
    Len1 = auto()
    Len2 = auto()
    Len3 = auto()
    Ecl1 = auto()
    Ecl1x = auto()
    Ecl2 = auto()
    Ecl3 = auto()
    PrefT = auto()
    PrefH = auto()
    Len1D = auto()
    Len2D = auto()
    Len3D = auto()


class Strength:
    Strong = auto()
    Weak = auto()


class Number:
    Sg = auto()
    Pl = auto()


class Gender:
    Masc = auto()
    Fem = auto()


# Encapsulates a word form, a phrase form or a clause form:
@dataclass
class Form:
    value: str


# Class for noun and noun phrase forms in the singular:
class FormSg(Form):
    gender: Gender


# Class for noun forms in the plural genitive:
class FormPlGen(Form):
    strength: Strength
    # in the plural genitive, a noun form has strength.
