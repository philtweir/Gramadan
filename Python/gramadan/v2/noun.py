from __future__ import annotations

from lxml import etree as ET
from typing import Optional, Union

from gramadan import noun
from .features import FormSg, Form, FormPlGen, Gender, Strength
from .singular_info import SingularInfo
from .plural_info import PluralInfo
from .entity import Entity
from .opers import Opers
noun.SingularInfo = SingularInfo
noun.PluralInfo = PluralInfo
noun.Form = Form

# A noun
class Noun(Entity[noun.Noun]):
    super_cls = noun.Noun

    _form_fields = (
        'sgNom',
        'sgGen',
        'sgVoc',
        'sgDat',
        'plNom',
        'plGen',
        'plVoc',
        'count',
    )

    # Constructors:
    @classmethod
    def create_from_info(cls, si: SingularInfo, pi: Optional[PluralInfo] = None) -> Noun:
        v1 = noun.Noun.create_from_info(si, pi)
        return cls(v1=v1)

    @classmethod
    def create_from_conllu(cls, token, default_nom_case: bool=False) -> Noun:
        if token.upos not in ("NOUN", "PROPN"):
            raise RuntimeError("Attempting to convert a CoNLL non-noun to a (partial) noun")

        try:
            case = token.feats.get("Case", {"Nom"}) if default_nom_case else token.feats["Case"]
        except KeyError:
            raise RuntimeError("Must know case of a CoNLL noun to fill any fields")
        case = next(iter(case))

        si = SingularInfo(None, [], [], [], [])
        pi = PluralInfo(None, [], [], [])
        form = Opers.Demutate(token.form)

        if "Plur" in token.feats.get("Number", {"Sing"}):
            if case == "Gen":
                strength = getattr(Strength, next(iter(token.feats["NounType"]))) \
                        if token.feats.get("NounType", None) else None
                pi.strength = strength
                pi.genitive.append(FormPlGen(value=form, strength=strength)) # Unknown
            else:
                pi.nominative.append(Form(value=form))
        else:
            gender = token.feats.get("Gender", {})
            gender = Gender.Masc if "Masc" in gender else \
                Gender.Fem if "Fem" in gender else None
            si.gender = gender
            if case == "Gen":
                si.genitive.append(FormSg(value=form, gender=gender))
            else:
                si.nominative.append(FormSg(value=form, gender=gender))

        noun = cls.create_from_info(si, pi)
        noun.is_proper = token.upos == "PROPN"
        noun.is_definite = "Def" in token.feats.get("Definite", {})
        return noun

    @classmethod
    def from_str(
        cls,
        *args,
        **kwargs
    ) -> Noun:
        v1 = noun.Noun.from_str(*args, **kwargs)
        return cls(v1=v1)

    @property
    def gender(self):
        return self.v1.getGender()

    @property
    def is_proper(self):
        return self.v1.isProper

    @is_proper.setter
    def is_proper(self, proper: bool):
        self.v1.isProper = proper

    @property
    def is_definite(self):
        return self.v1.isProper

    @is_definite.setter
    def is_definite(self, definite: bool):
        self.v1.isDefinite = definite

    @property
    def is_immutable(self):
        return self.v1.isImmutable

    @is_immutable.setter
    def is_immutable(self, immutable: bool):
        self.v1.isImmutable = immutable
