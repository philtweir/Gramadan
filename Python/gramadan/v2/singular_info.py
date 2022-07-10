from typing import Callable
from gramadan.singular_info import (
    SingularInfoV1,
    SingularInfoO   as SingularInfoOV1,
    SingularInfoC   as SingularInfoCV1,
    SingularInfoL   as SingularInfoLV1,
    SingularInfoE   as SingularInfoEV1,
    SingularInfoA   as SingularInfoAV1,
    SingularInfoD   as SingularInfoDV1,
    SingularInfoN   as SingularInfoNV1,
    SingularInfoEAX as SingularInfoEAXV1,
    SingularInfoAX  as SingularInfoAXV1
)

from .features import FormSg, Form, FormPlGen, Gender

# FIXME
# IMPORTANT NOTE: the original C# tester does not seem
# to cover these classes, so Python testing is non-existent
# for these so far.

# A class that encapsulates the singular forms of a noun or adjective:
class SingularInfo(SingularInfoV1):
    v1 = None
    super_type: Callable = SingularInfoV1

    def __getattribute__(self, attr):
        return getattr(self.v1, attr)

    def __setattribute__(self, attr, val):
        return setattr(self.v1, attr, val)

    # Forms of the verb:
    def __init__(
        self,
        *args,
        v1: SingularInfoV1 = None,
        **kwargs
    ):
        if v1:
            self.v1 = v1
        else:
            self.v1 = self.super_type(*args, **kwargs)


# Singular class O: all cases are identical.
class SingularInfoO(SingularInfo):
    super_type = SingularInfoOV1

# Singular class C: genitive and vocative formed by slenderization.
class SingularInfoC(SingularInfo):
    super_type = SingularInfoCV1


# Singular class L: genitive formed by broadening.
class SingularInfoL(SingularInfo):
    super_type = SingularInfoLV1


# Singular class E: genitive formed by suffix "-e".
class SingularInfoE(SingularInfo):
    super_type = SingularInfoEV1


# Singular class A: genitive formed by suffix "-a".
class SingularInfoA(SingularInfo):
    super_type = SingularInfoAV1


# Singular class D: genitive ends in "-d".
class SingularInfoD(SingularInfo):
    super_type = SingularInfoDV1


# Singular class N: genitive ends in "-n".
class SingularInfoN(SingularInfo):
    super_type = SingularInfoNV1


# Singular class EAX: genitive formed by suffix "-each".
class SingularInfoEAX(SingularInfo):
    super_type = SingularInfoEAXV1


# Singular class AX: genitive formed by suffix "-ach".
class SingularInfoAX(SingularInfo):
    super_type = SingularInfoAXV1
