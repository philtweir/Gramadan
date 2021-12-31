from typing import Optional
from enum import Enum, auto
from features import Form

# Enumerations used to access verb phrase forms:
class VPTense(Enum):
    Any = auto()
    Past = auto()
    PastCont = auto()
    Pres = auto()
    PresCont = auto()
    Fut = auto()
    Cond = auto()


class VPMood(Enum):
    Imper = auto()
    Subj = auto()


class VPShape(Enum):
    Any = auto()
    Declar = auto()
    Interrog = auto()
    # /*, RelDep, RelIndep, Report*/ }


class VPPerson(Enum):
    Any = auto()
    Sg1 = auto()
    Sg2 = auto()
    Sg3Masc = auto()
    Sg3Fem = auto()
    Pl1 = auto()
    Pl2 = auto()
    Pl3 = auto()
    NoSubject = auto()
    Auto = auto()


class VPPolarity(Enum):
    Any = auto()
    Pos = auto()
    Neg = auto()


VPTenseDictionary = dict[
    VPTense, dict[VPShape, dict[VPPerson, dict[VPPolarity, list[Form]]]]
]
VPMoodDictionary = dict[VPMood, dict[VPPerson, dict[VPPolarity, list[Form]]]]

# A verbal phrase:
class VP:
    def __init__(
        self, tenses: Optional[VPTenseDictionary], moods: Optional[VPMoodDictionary]
    ):
        # Forms of the verbal phrase:
        self.tenses: VPTenseDictionary = {} if tenses is None else tenses
        self.moods: VPMoodDictionary = {} if moods is None else moods

    # Constructs a verbal phrase from a verb:
    @classmethod
    def from_verb(v: "Verb") -> "VP":
        # region prepare-structure
        ts: tuple[VPTense] = (
            VPTense.Past,
            VPTense.PastCont,
            VPTense.Pres,
            VPTense.PresCont,
            VPTense.Fut,
            VPTense.Cond,
        )
        ms: tuple[VPMood] = (VPMood.Imper, VPMood.Subj)
        ss: tuple[VPShape] = (
            VPShape.Declar,
            VPShape.Interrog,
        )  # , VPShape.RelDep, VPShape.RelIndep, VPShape.Report*
        pers: tuple[VPPerson] = (
            VPPerson.Sg1,
            VPPerson.Sg2,
            VPPerson.Sg3Masc,
            VPPerson.Sg3Fem,
            VPPerson.Pl1,
            VPPerson.Pl2,
            VPPerson.Pl3,
            VPPerson.NoSubject,
            VPPerson.Auto,
        )
        pols: tuple[VPPolarity] = (VPPolarity.Pos, VPPolarity.Neg)

        tenses: VPTenseDictionary = {}
        t: VPTense
        for t in ts:
            tenses[t] = {}
            s: VPShape
            for s in ss:
                tenses[t][s] = {}
                per: VPPerson
                for per in pers:
                    tenses[t][s][per] = {}
                    pol: VPPolarity
                    for pol in pols:
                        this.tenses[t][s][per][pol] = []

        moods: VPMoodDictionary = {}
        m: VPMood
        for m in ms:
            moods[m] = {}
            per: VPPerson
            for per in pers:
                moods[m][per] = {}
                pol: VPPolarity
                for pol in pols:
                    moods[m][per][pol] = []
        # endregion

        # Apply rules to build tensed forms:
        t: VPTense
        for t in v.tenseRules:
            p: VPPerson
            for p in v.tenseRules[t]:
                s: VPShape
                for s in v.tenseRules[t][p]:
                    l: VPPolarity
                    for l in v.tenseRules[t][p][s]:
                        rule: VerbTenseRule
                        for rule in v.tenseRules[t][p][s][l]:
                            # For each verb form, use the rule to build a verbal phrase form:
                            vForm: Form
                            for vForm in v.tenses[rule.verbTense][rule.verbDependency][
                                rule.verbPerson
                            ]:
                                vpForm: Form = Form("")
                                if rule.particle != "":
                                    vpForm.value += rule.particle + " "
                                vpForm.value += Opers.Mutate(rule.mutation, vForm.value)
                                if rule.pronoun != "":
                                    vpForm.value += " " + rule.pronoun
                                if (
                                    v.getLemma() == "bí"
                                    and t == VPTense.Pres
                                    and s == VPShape.Declar
                                    and l == VPPolarity.Neg
                                    and vpForm.value.StartsWith("ní fhuil")
                                ):
                                    vpForm.value = "níl" + vpForm.value.Substring(8)
                                    # ní fhuil --> níl
                                tenses[t][s][p][l].append(vpForm)

        # region mappings
        mapPerson: dict[VPPerson, VerbPerson] = {}
        mapPerson[VPPerson.Sg1] = VerbPerson.Sg1
        mapPerson[VPPerson.Sg2] = VerbPerson.Sg2
        mapPerson[VPPerson.Sg3Masc] = VerbPerson.Sg3
        mapPerson[VPPerson.Sg3Fem] = VerbPerson.Sg3
        mapPerson[VPPerson.Pl1] = VerbPerson.Pl1
        mapPerson[VPPerson.Pl2] = VerbPerson.Pl2
        mapPerson[VPPerson.Pl3] = VerbPerson.Pl3
        mapPerson[VPPerson.NoSubject] = VerbPerson.Base
        mapPerson[VPPerson.Auto] = VerbPerson.Auto
        mapPronoun: dict[VPPerson, str] = {}
        mapPronoun[VPPerson.Sg1] = " mé"
        mapPronoun[VPPerson.Sg2] = " tú"
        mapPronoun[VPPerson.Sg3Masc] = " sé"
        mapPronoun[VPPerson.Sg3Fem] = " sí"
        mapPronoun[VPPerson.Pl1] = " muid"
        mapPronoun[VPPerson.Pl2] = " sibh"
        mapPronoun[VPPerson.Pl3] = " siad"
        mapPronoun[VPPerson.NoSubject] = ""
        mapPronoun[VPPerson.Auto] = ""
        # endregion
        # region create-mood-imperative
        vpPerson: VPPerson
        for vpPerson in pers:
            hasSyntheticForms: bool = False
            # Synthetic forms:
            vForm: Form
            for vForm in v.moods[VerbMood.Imper][mapPerson[vpPerson]]:
                pos: str = vForm.value
                neg: str = "ná " + Opers.Mutate(Mutation.PrefH, vForm.value)
                moods[VPMood.Imper][vpPerson][VPPolarity.Pos].append(Form(pos))
                moods[VPMood.Imper][vpPerson][VPPolarity.Neg].append(Form(neg))
                hasSyntheticForms = True

            # Analytic forms:
            if (
                not hasSyntheticForms
                or vpPerson == VPPerson.Pl1
                or vpPerson == VPPerson.Pl3
            ):
                vForm: Form
                for vForm in v.moods[VerbMood.Imper][VerbPerson.Base]:
                    pos: str = vForm.value + mapPronoun[vpPerson]
                    neg: str = (
                        "ná "
                        + Opers.Mutate(Mutation.PrefH, vForm.value)
                        + mapPronoun[vpPerson]
                    )
                    moods[VPMood.Imper][vpPerson][VPPolarity.Pos].append(Form(pos))
                    moods[VPMood.Imper][vpPerson][VPPolarity.Neg].append(Form(neg))
                    hasSyntheticForms = True

        # endregion
        # region create-mood-subjunctive
        vpPerson: VPPerson
        for vpPerson in pers:
            posMut: Mutation = Mutation.Ecl1
            negMut: Mutation = Mutation.Len1
            negParticle: str = "nár "

            # Exceptions for irregular verbs:
            if v.getLemma() == "abair":
                negMut = Mutation.Nil
            if v.getLemma() == "bí":
                negParticle = "ná "

            hasSyntheticForms: bool = False
            # Synthetic forms:
            vForm: Form
            for vForm in v.moods[VerbMood.Subj][mapPerson[vpPerson]]:
                pos: str = "go " + Opers.Mutate(posMut, vForm.value)
                neg: str = negParticle + Opers.Mutate(negMut, vForm.value)
                moods[VPMood.Subj][vpPerson][VPPolarity.Pos].append(Form(pos))
                moods[VPMood.Subj][vpPerson][VPPolarity.Neg].append(Form(neg))
                hasSyntheticForms = True

            # Analytic forms:
            if not hasSyntheticForms or vpPerson == VPPerson.Pl1:
                vForm: Form
                for vForm in v.moods[VerbMood.Subj][VerbPerson.Base]:
                    pos: str = (
                        "go " + Opers.Mutate(posMut, vForm.value) + mapPronoun[vpPerson]
                    )
                    neg: str = (
                        negParticle
                        + Opers.Mutate(negMut, vForm.value)
                        + mapPronoun[vpPerson]
                    )
                    moods[VPMood.Subj][vpPerson][VPPolarity.Pos].append(Form(pos))
                    moods[VPMood.Subj][vpPerson][VPPolarity.Neg].append(Form(neg))
                    hasSyntheticForms = true
        # endregion
        return cls(tenses=tenses, moods=moods)

    # Prints a user-friendly summary of the verbal phrase in one of its tenses, shapes and polarities:
    def print_by_tense(tense: VPTense, shape: VPShape, pol: VPPolarity) -> str:
        ret: str = ""
        pers: tuple(VPPerson) = (
            VPPerson.Sg1,
            VPPerson.Sg2,
            VPPerson.Sg3Masc,
            VPPerson.Sg3Fem,
            VPPerson.Pl1,
            VPPerson.Pl2,
            VPPerson.Pl3,
            VPPerson.NoSubject,
            VPPerson.Auto,
        )

        per: VPPerson
        for per in pers:
            ret += per.ToString() + ": "
            f: Form
            for f in this.tenses[tense][shape][per][pol]:
                ret += "[" + f.value + "] "
            ret += "\n"
        return ret

    # Prints a user-friendly summary of the verbal phrase in one of its moods and polarities:
    def print_by_mood(mood: VPMood, pol: VPPolarity):
        ret: str = ""
        pers: tuple(VPPerson) = (
            VPPerson.Sg1,
            VPPerson.Sg2,
            VPPerson.Sg3Masc,
            VPPerson.Sg3Fem,
            VPPerson.Pl1,
            VPPerson.Pl2,
            VPPerson.Pl3,
            VPPerson.NoSubject,
            VPPerson.Auto,
        )
        per: VPPerson
        for per in pers:
            ret += per.ToString() + ": "
            f: Form
            for f in this.moods[mood][per][pol]:
                ret += "[" + f.value + "] "
            ret += "\n"
        return ret
