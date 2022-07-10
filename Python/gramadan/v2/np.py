from __future__ import annotations

from lxml import etree as ET
from typing import Optional, Union
from gramadan import np
from gramadan.possessive import Possessive
from .features import Form
from .noun import Noun
from .adjective import Adjective
from .entity import Entity

np.NounType = Noun # type: ignore
np.Form = Form

# A class for a noun phrase:
class NP(Entity[np.NP]):
    super_cls = np.NP

    _form_fields = (
        'sgNom',
        'sgGen',
        'sgDat',
        'sgNomArt',
        'sgGenArt',
        'sgDatArtN',
        'sgDatArtS',
        'plNom',
        'plGen',
        'plDat',
        'plNomArt',
        'plGenArt',
        'plDatArt',
    )

    # Creates a noun phrase from an explicit listing of all the basic forms:
    @classmethod
    def create(
        cls,
        *args,
        **kwargs
    ):
        v1 = cls.super_cls.create(*args, **kwargs)
        return cls(v1=v1)

    # Creates a noun phrase from a noun determined by a possessive pronoun:
    @classmethod
    def create_from_possessive(cls, head: Noun, poss: Possessive) -> NP:
        v1 = cls.super_cls.create_from_possessive(head, poss) # type: ignore
        return cls(v1=v1)

    # Creates a noun phrase from a noun modified by an adjective determined by a possessive pronoun:
    @classmethod
    def create_from_noun_adjective_possessive(
        cls, head: Noun, mod: Adjective, poss: Possessive
    ) -> NP:
        v1 = cls.super_cls.create_from_noun_adjective_possessive(head, mod, poss) # type: ignore
        return cls(v1=v1)

    # Creates a noun phrase from a noun:
    @classmethod
    def create_from_noun(cls, head: Noun) -> NP:
        v1 = cls.super_cls.create_from_noun(head) # type: ignore
        return cls(v1=v1)

    # Creates a noun phrase from a noun modified by an adjective:
    @classmethod
    def create_from_noun_adjective(cls, head: Noun, mod: Adjective) -> NP:
        v1 = cls.super_cls.create_from_noun_adjective(head, mod) # type: ignore
        return cls(v1=v1)
