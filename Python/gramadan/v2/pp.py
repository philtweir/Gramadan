from __future__ import annotations

from lxml import etree as ET
from typing import Optional, Union
from gramadan import pp
from gramadan.possessive import Possessive
from .features import Form
from .noun import Noun
from .adjective import Adjective
from .entity import Entity
from .np import NP

pp.Form = Form

# A class for a noun phrase:
class PP(Entity[pp.PP]):
    super_cls = pp.PP

    _form_fields = (
        'sg',
        'sgArtN',
        'sgArtS',
        'pl',
        'plArt',
    )

    # Creates a noun phrase from an explicit listing of all the basic forms:
    @classmethod
    def create(cls, prep: Preposition, np: NP) -> "PP":
        v1 = cls.super_cls.create(prep, np)
        return cls(v1=v1)
