import re
from dataclasses import dataclass, field
from typing import Callable
from gramadan.plural_info import (
    PluralInfoV1,
    PluralInfoLgC as PluralInfoLgCV1,
    PluralInfoLgE as PluralInfoLgEV1,
    PluralInfoLgA as PluralInfoLgAV1,
    PluralInfoTr  as PluralInfoTrV1,
)
from .features import FormSg, Form, FormPlGen, Strength

# FIXME
# IMPORTANT NOTE: the original C# tester does not seem
# to cover these classes, so Python testing is non-existent
# for these so far.

@dataclass
class PluralInfo(PluralInfoV1):
    v1 = None
    super_type: Callable = PluralInfoV1

    def __getattr__(self, attr):
        return getattr(self.v1, attr)

    def __setattribute__(self, attr, val):
        return setattr(self.v1, attr, val)

    # Forms of the verb:
    def __init__(
        self,
        *args,
        v1: PluralInfoV1 = None,
        **kwargs
    ):
        if v1:
            self.v1 = v1
        else:
            self.v1 = self.super_type(*args, **kwargs)

# Plural class LgC: weak, plural formed by slenderization.
class PluralInfoLgC(PluralInfo):
    super_type = PluralInfoLgCV1


# Plural class LgE: weak, plural formed by suffix "-e".
class PluralInfoLgE(PluralInfo):
    super_type = PluralInfoLgEV1


# Plural class LgA: weak, plural formed by suffix "-a".
class PluralInfoLgA(PluralInfo):
    super_type = PluralInfoLgAV1

# Plural class Tr: strong.
class PluralInfoTr(PluralInfo):
    super_type = PluralInfoTrV1
