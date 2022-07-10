from __future__ import annotations

from lxml import etree as ET
from typing import Optional, Union

from gramadan import noun
from .features import FormSg, Form, FormPlGen, Gender, Strength
from .singular_info import SingularInfo
from .plural_info import PluralInfo
from .entity import Entity
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
    def from_str(
        cls,
        *args,
        **kwargs
    ) -> Noun:
        v1 = noun.Noun.from_str(*args, **kwargs)
        return cls(v1=v1)
