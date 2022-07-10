from __future__ import annotations

from lxml import etree as ET
import re
from typing import List, Optional, Union
from gramadan import adjective
from .singular_info import SingularInfo
from .features import Form
from .entity import Entity
adjective.SingularInfo = SingularInfo
adjective.Form = Form


class Adjective(Entity[adjective.Adjective]):
    super_cls = adjective.Adjective

    _form_fields = (
        'sgNom',
        'sgGenMasc',
        'sgGenFem',
        'sgVocMasc',
        'sgVocFem',
        'plNom',
        'graded',
        'abstractNoun',
    )

    @classmethod
    def create_from_singular_info(
        cls,
        sgMasc: SingularInfo,
        sgFem: SingularInfo,
        plural_or_graded: Optional[str],
        graded: Optional[str],
    ) -> Adjective:
        v1 = adjective.Adjective.create_from_singular_info(sgMasc, sgFem, plural_or_graded, graded)
        return cls(v1=v1)
