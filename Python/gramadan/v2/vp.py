from __future__ import annotations

from lxml import etree as ET
from typing import Optional, Union
from gramadan import vp
from gramadan.possessive import Possessive
from .verb import Verb, VPShape
from .features import Form
from .entity import Entity

vp.VerbType = Verb # type: ignore
vp.VPShapeType = VPShape
vp.Form = Form

# A class for a noun phrase:
class VP(Entity[vp.VP]):
    super_cls = vp.VP

    _form_fields = ()

    # Creates a verbal phrase from a verb:
    @classmethod
    def from_verb(cls, v: Verb) -> VP:
        v1 = cls.super_cls.from_verb(v) # type: ignore
        return cls(v1=v1)
