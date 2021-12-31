from dataclasses import dataclass, field
from features import FormSg, Form, FormPlGen, Strength


@dataclass
class PluralInfo:
    strength: Strength
    nominative: list[Form] = field(default_factory=list)
    genitive: list[Form] = field(default_factory=list)
    vocative: list[Form] = field(default_factory=list)

    def print() -> str:
        ret: str = ""
        ret += "NOM: "
        f: Form
        for f in self.nominative:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "GEN: "
        f: Form
        for f in self.genitive:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "VOC: "
        f: Form
        for f in self.vocative:
            ret += "[" + f.value + "] "
        ret += "\n"
        return ret


# Plural class LgC: weak, plural formed by slenderization.
class PluralInfoLgC(PluralInfo):
    def __init__(self, bayse: str, slenderizationTarget: str = ""):
        super().__init__(strength=Strength.Weak)

        # generate the genitive:
        form: str = Opers.Broaden(bayse)
        self.genitive.append(Form(form))

        # generate the vocative:
        form = form + "a"
        self.vocative.append(Form(form))

        # generate the nominative:
        form = bayse
        form = Regex.Replace(form, "ch$", "gh")
        # eg. bacach > bacaigh
        form = Opers.Slenderize(form, slenderizationTarget)
        self.nominative.append(Form(form))


# Plural class LgE: weak, plural formed by suffix "-e".
class PluralInfoLgE(PluralInfo):
    def __init__(self, bayse: str, slenderizationTarget: str = ""):
        super().__init__(strength=Strength.Weak)

        form: str = bayse
        form = Opers.Slenderize(form, slenderizationTarget) + "e"

        self.nominative.append(Form(form))
        self.genitive.append(Form(Opers.Broaden(bayse)))
        self.vocative.append(Form(form))


# Plural class LgA: weak, plural formed by suffix "-a".
class PluralInfoLgA(PluralInfo):
    def __init__(bayse: str, broadeningTarget: str = ""):
        super().__init__(strength=Strength.Weak)

        form: str = bayse
        form = Opers.Broaden(form, broadeningTarget) + "a"

        self.nominative.append(Form(form))
        self.genitive.append(Form(Opers.Broaden(bayse)))
        self.vocative.append(Form(form))


# Plural class Tr: strong.
class PluralInfoTr(PluralInfo):
    def __init__(bayse: str):
        super().__init__(strength=Strength.Strong)
        self.nominative.append(Form(bayse))
        self.genitive.append(Form(bayse))
        self.vocative.append(Form(bayse))
