from typing import Optional, Sequence
from collections import namedtuple
from gramadan.verb import VerbTenseRule, VerbTense, VerbMood, VerbPerson
from gramadan.features import Mutation
from .entity import Entity
from .copula import Copula
from .features import Form, Number, Gender
from .verb import VPPerson, VPShape, VPPolarity
from .opers import Opers
from .np import NP


CNPTenseDictionary = dict[VerbTense,
    dict[VPShape,
        dict[VPPolarity,
            dict[str | tuple[str, str], list[Form]]
        ]
    ]
]
CNPMoodDictionary = dict[VerbMood, dict[VPPolarity, dict[str | tuple[str, str], list[Form]]]]
CNPPronounTuple = namedtuple("CNPPronounTuple", ["subj", "subjEmph", "obj", "objEmph"])

# A copular noun phrase:
class CNP(Entity["CNP"]):
    def __init__(
        self, tenses: Optional[CNPTenseDictionary], moods: Optional[CNPMoodDictionary]
    ):
        # Forms of the verbal phrase:
        self.tenses: CNPTenseDictionary = {} if tenses is None else tenses
        self.moods: CNPMoodDictionary = {} if moods is None else moods

    # Constructs a copular noun phrase from a noun.
    @classmethod
    def create_from_noun_phrases(cls, copula: Copula, subj: NP | VPPerson, prd: NP | VPPerson) -> "CNP":
        # Note that this doesn't cover reported clauses, e.g. is é mo bharúil go...

        if isinstance(subj, VerbPerson) or isinstance(prd, VerbPerson):
            raise NotImplementedError("Use VPPerson not VerbPerson")

        # region prepare-structure
        ts: Sequence[VerbTense] = (
            VerbTense.Past,
            VerbTense.PastCont,
            VerbTense.Pres,
            VerbTense.PresCont,
            VerbTense.Fut,
            VerbTense.Cond,
        )
        ss: Sequence[VPShape] = tuple(VPShape.__members__.values())  # , VPShape.RelIndep, VPShape.Report*
        ms: Sequence[VerbMood] = (VerbMood.Subj,)
        pols: Sequence[VPPolarity] = (VPPolarity.Pos, VPPolarity.Neg)

        tenses: CNPTenseDictionary = {}
        t: VerbTense
        s: VPShape

        mapPronoun: dict[VPPerson, CNPPronounTuple] = {}

        mapPronoun[VPPerson.Sg1] = CNPPronounTuple("mé", "mise", "mé", "mise")
        mapPronoun[VPPerson.Sg2] = CNPPronounTuple("tú", "tusa", "thu", "thusa")
        mapPronoun[VPPerson.Sg3Masc] = CNPPronounTuple("sé", "seisean", "é", "eisean")
        mapPronoun[VPPerson.Sg3Fem] = CNPPronounTuple("sí", "sise", "í", "íse")
        mapPronoun[VPPerson.Pl1] = CNPPronounTuple("muid", "muidne", "muid", "muidne")
        mapPronoun[VPPerson.Pl2] = CNPPronounTuple("sibh", "sibhse", "sibh", "sibhse")
        mapPronoun[VPPerson.Pl3] = CNPPronounTuple("siad", "siadsan", "iad", "iadsan")

        def _build_form(*parts: None | Form | tuple[Form, Form | None]) -> Form:
            ret = ""
            for prv, nxt in zip(parts[:-1], parts[1:]):
                if isinstance(prv, tuple):
                    if isinstance(nxt, Form) and Opers.StartsVowelFhx(nxt.value):
                        vowel_form = prv[1] or prv[0]
                        ret += vowel_form.value
                    else:
                        ret += prv[0].value
                elif prv is None:
                    continue
                else:
                    ret += prv.value
                if ret[-1] != "'":
                    ret += " "
            if isinstance(nxt, tuple):
                ret += nxt[0].value
            elif nxt is not None:
                ret += nxt.value
            return Form(ret.strip())

        for t in ts:
            tenses[t] = {}
            for s in ss:
                tenses[t][s] = {}
                for pol in pols:
                    tenses[t][s][pol] = {}

                    if isinstance(prd, VPPerson) or isinstance(subj, VPPerson):
                        person: VPPerson
                        np: NP
                        person_is_subj: bool
                        if isinstance(prd, VPPerson) and isinstance(subj, VPPerson):
                            raise NotImplementedError("One part of copular phrase, at least, must not be a personal pronoun")
                        if not isinstance(prd, VPPerson):
                            np = prd
                            person = subj
                            person_is_subj = True
                        elif not isinstance(subj, VPPerson):
                            np = subj
                            person = prd
                            person_is_subj = False
                        forms = {
                            "definite": {Number.Sg: "sgNomArt", Number.Pl: "plNomArt"},
                            "indefinite": {Number.Sg: "sgNom", Number.Pl: "plNom"},
                        }
                        if np.isDefinite:
                            forms = {
                                "definite": {Number.Sg: "sgNom", Number.Pl: "plNom"},
                            }
                        for df, nforms in forms.items():
                            for n, form in nforms.items():
                                # Is mé an dochtúir
                                # Is mise an dochtúir
                                # Is é an dochtúir é
                                # Is eisean an dochtúir
                                suffix: Form | None
                                if prd in (VPPerson.Sg3Masc, VPPerson.Sg3Fem, VPPerson.Pl3) and person_is_subj and df == "indefinite":
                                    suffix = Form(mapPronoun[person].obj)
                                else:
                                    suffix = None
                                copula_tenses: list[tuple[Form, Form | None]] = copula.tenses.get(t, {}).get(s, {}).get(pol, [])
                                tenses[t][s][pol][form] = [
                                    _build_form(
                                        cop,
                                        Form(mapPronoun[person].obj if person_is_subj else mapPronoun[person].objEmph),
                                        sfm,
                                        suffix
                                    )
                                    for sfm in np.forms[form]
                                    for cop in copula_tenses
                                ]
                    else:
                        # We can assume two noun phrases.
                        pforms: dict[str, dict[str, dict[Number, str]]] = {}
                        for which, np in {"subj": subj, "prd": prd}.items():
                            pforms[which] = {
                                "definite": {Number.Sg: "sgNomArt", Number.Pl: "plNomArt"},
                                "indefinite": {Number.Sg: "sgNom", Number.Pl: "plNom"},
                            }
                            if np.isDefinite:
                                pforms[which] = {
                                    "definite": {Number.Sg: "sgNom", Number.Pl: "plNom"},
                                }

                        for p_df, p_nforms in forms.items():
                            for p_n, p_form in nforms.items():
                                for s_df, s_nforms in forms.items():
                                    for s_n, s_form in nforms.items():
                                        if p_n == Number.Pl:
                                            subpredicate = VPPerson.Pl3
                                        elif prd.hasGender() and prd.getGender() == Gender.Fem:
                                            subpredicate = VPPerson.Sg3Fem
                                        else:
                                            subpredicate = VPPerson.Sg3Masc

                                        copula_tenses: list[tuple[Form, Form | None]] = copula.tenses.get(t, {}).get(s, {}).get(pol, [])
                                        if p_df == "definite":
                                            # Identification
                                            if s_df == "definite":
                                                # Is é an dochtúir an bhean
                                                tenses[t][s][pol][(p_form, s_form)] = [
                                                    _build_form(
                                                        cop,
                                                        Form(mapPronoun[subpredicate].obj),
                                                        pfm,
                                                        sfm
                                                    )
                                                    for pfm in prd.forms[p_form]
                                                    for sfm in subj.forms[s_form]
                                                    for cop in copula_tenses
                                                ]
                                            else:
                                                # A definite predicate => a definite subject
                                                ...
                                        else:
                                            # Classifactorial
                                            if s_df == "definite":
                                                if s_n == Number.Pl:
                                                    subsubject = VPPerson.Pl3
                                                elif subj.hasGender() and subj.getGender() == Gender.Fem:
                                                    subsubject = VPPerson.Sg3Fem
                                                else:
                                                    subsubject = VPPerson.Sg3Masc

                                                # Is dochtúir í an bhean
                                                tenses[t][s][pol][(p_form, s_form)] = sum((
                                                    [
                                                        _build_form(
                                                            cop,
                                                            pfm,
                                                            Form(mapPronoun[subsubject].obj),
                                                            sfm
                                                        ),
                                                        _build_form(
                                                            pfm,
                                                            cop,
                                                            Form("ea"),
                                                            sfm
                                                        )
                                                    ]
                                                    for pfm in prd.forms[p_form]
                                                    for sfm in subj.forms[s_form]
                                                    for cop in copula_tenses
                                                ), [])
                                            else:
                                                # Is dochtúir bean
                                                tenses[t][s][pol][(p_form, s_form)] = [
                                                    _build_form(
                                                        cop,
                                                        pfm,
                                                        sfm
                                                    )
                                                    for pfm in prd.forms[p_form]
                                                    for sfm in subj.forms[s_form]
                                                    for cop in copula_tenses
                                                ]


        moods: CNPMoodDictionary = {}
        for m in ms:
            moods[t] = {}
            for pol in pols:
                moods[t][pol] = []
        # endregion
        return cls(tenses=tenses, moods=moods)

    # Prints a user-friendly summary of the verbal phrase in one of its tenses, shapes and polarities:
    def print_by_tense(self, tense: VerbTense, shape: VPShape, pol: VPPolarity, sub_per: VPPerson, obj_per: VPPerson = VPPerson.NoSubject) -> str:
        ret: str = ""
        for f in self.tenses[tense][shape][pol][sub_per][obj_per]:
            ret += "[" + f.value + "] "
        ret += "\n"
        return ret
