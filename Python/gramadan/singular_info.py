import re
from dataclasses import dataclass, field
from .opers import Opers
from .features import FormSg, Form, FormPlGen, Gender

# FIXME
# IMPORTANT NOTE: the original C# tester does not seem
# to cover these classes, so Python testing is non-existent
# for these so far.

# A class that encapsulates the singular forms of a noun or adjective:
@dataclass
class SingularInfo:
    gender: Gender
    nominative: list[Form] = field(default_factory=list)
    genitive: list[Form] = field(default_factory=list)
    vocative: list[Form] = field(default_factory=list)
    dative: list[Form] = field(default_factory=list)

    def print(self) -> str:
        ret: str = ""
        ret += "NOM: "
        f: Form
        for f in self.nominative:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "GEN: "

        for f in self.genitive:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "VOC: "

        for f in self.vocative:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "DAT: "

        for f in self.dative:
            ret += "[" + f.value + "] "
        ret += "\n"
        return ret


# Singular class O: all cases are identical.
class SingularInfoO(SingularInfo):
    def __init__(self, lemma: str, gender: Gender):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.genitive.append(Form(lemma))
        self.vocative.append(Form(lemma))
        self.dative.append(Form(lemma))


# Singular class C: genitive and vocative formed by slenderization.
class SingularInfoC(SingularInfo):
    def __init__(self, lemma: str, gender: Gender, slenderizationTarget: str = ""):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.dative.append(Form(lemma))

        # derive and assign the vocative:
        form: str = lemma
        form = re.sub("ch$", "gh", form)
        # eg. bacach > bacaigh
        form = Opers.SlenderizeWithTarget(form, slenderizationTarget)
        if gender == Gender.Fem:
            self.vocative.append(Form(lemma))
        else:
            self.vocative.append(Form(form))

        # derive and assign the genitive:
        if gender == Gender.Fem:
            form = re.sub("igh$", "í", form)  # eg. cailleach > cailleaí
        self.genitive.append(Form(form))


# Singular class L: genitive formed by broadening.
class SingularInfoL(SingularInfo):
    def __init__(self, lemma: str, gender: Gender, broadeningTarget: str = ""):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.vocative.append(Form(lemma))
        self.dative.append(Form(lemma))

        # derive the genitive:
        form: str = lemma
        form = Opers.BroadenWithTarget(form, broadeningTarget)
        self.genitive.append(Form(form))


# Singular class E: genitive formed by suffix "-e".
class SingularInfoE(SingularInfo):
    def __init__(
        self,
        lemma: str,
        gender: Gender,
        syncope: bool,
        doubleDative: bool,
        slenderizationTarget: str = "",
    ):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.vocative.append(Form(lemma))

        # derive the dative:
        form: str = lemma
        if syncope:
            form = Opers.Syncope(form)
        form = Opers.SlenderizeWithTarget(form, slenderizationTarget)
        if not doubleDative:
            self.dative.append(Form(lemma))
        else:
            self.dative.append(Form(lemma))
            self.dative.append(Form(form))

        # continue deriving the genitive:
        form = re.sub("([" + Opers.VowelsSlender + "])ngt$", r"\1ngth", form)
        # eg. tarraingt > tarraingthe
        form = re.sub("ú$", "aith", form)
        # eg. scrúdú > scrúdaithe
        form = form + "e"
        self.genitive.append(Form(form))


# Singular class A: genitive formed by suffix "-a".
class SingularInfoA(SingularInfo):
    def __init__(
        self, lemma: str, gender: Gender, syncope: bool, broadeningTarget: str = ""
    ):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.vocative.append(Form(lemma))
        self.dative.append(Form(lemma))

        # derive the genitive:
        form: str = lemma
        form = re.sub("([" + Opers.VowelsSlender + "])rt$", r"\1rth", form)
        # eg. bagairt > bagartha
        form = re.sub("([" + Opers.VowelsSlender + "])nnt$", r"\1nn", form)
        # eg. cionnroinnt > cionnranna
        form = re.sub("([" + Opers.VowelsSlender + "])nt$", r"\1n", form)
        # eg. canúint > canúna
        if syncope:
            form = Opers.Syncope(form)
        form = Opers.BroadenWithTarget(form, broadeningTarget)
        form = form + "a"
        self.genitive.append(Form(form))


# Singular class D: genitive ends in "-d".
class SingularInfoD(SingularInfo):
    def __init__(self, lemma: str, gender: Gender):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.vocative.append(Form(lemma))
        self.dative.append(Form(lemma))

        # derive the genitive:
        form: str = lemma
        form = re.sub("([" + Opers.VowelsBroad + "])$", r"\1d", form)
        # eg. cara > carad
        form = re.sub("([" + Opers.VowelsSlender + "])$", r"\1ad", form)
        # eg. fiche > fichead
        self.genitive.append(Form(form))


# Singular class N: genitive ends in "-n".
class SingularInfoN(SingularInfo):
    def __init__(self, lemma: str, gender: Gender):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.vocative.append(Form(lemma))
        self.dative.append(Form(lemma))

        # derive the genitive:
        form: str = lemma
        form = re.sub("([" + Opers.VowelsBroad + "])$", r"\1n", form)
        form = re.sub("([" + Opers.VowelsSlender + "])$", r"\1an", form)
        self.genitive.append(Form(form))


# Singular class EAX: genitive formed by suffix "-each".
class SingularInfoEAX(SingularInfo):
    def __init__(
        self, lemma: str, gender: Gender, syncope: bool, slenderizationTarget: str = ""
    ):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.vocative.append(Form(lemma))
        self.dative.append(Form(lemma))

        # derive the genitive:
        form: str = lemma
        if syncope:
            form = Opers.Syncope(form)
        form = Opers.SlenderizeWithTarget(form, slenderizationTarget)
        form = form + "each"
        self.genitive.append(Form(form))


# Singular class AX: genitive formed by suffix "-ach".
class SingularInfoAX(SingularInfo):
    def __init__(
        self, lemma: str, gender: Gender, syncope: bool, broadeningTarget: str = ""
    ):
        super().__init__(gender=gender)
        self.nominative.append(Form(lemma))
        self.vocative.append(Form(lemma))
        self.dative.append(Form(lemma))

        # derive the genitive:
        form: str = lemma
        if syncope:
            form = Opers.Syncope(form)
        form = Opers.BroadenWithTarget(form, broadeningTarget)
        form = form + "ach"
        self.genitive.append(Form(form))
