from __future__ import annotations

from typing import Optional, Union, Sequence
from lxml import etree as ET
from gramadan.verb import VerbMood, VerbTense, VPPolarity
from .features import Form
from .verb import VPShape
from .entity import Entity

# A pronoun:
class Pronoun(Entity["Pronoun"]):
    _form_fields = (
        'tenses_flattened',
    )

    disambig: str = ""

    # Forms of the verb:
    def __init__(
        self,
        tenses: Optional[TenseDictionary] = None,
        moods: Optional[MoodDictionary] = None,
        disambig: str = "",
    ):
        # region prepare-structure-for-data
        ts: Sequence[VerbTense] = (
            VerbTense.Past,
            VerbTense.PastCont,
            VerbTense.Pres,
            VerbTense.PresCont,
            VerbTense.Fut,
            VerbTense.Cond,
        )
        # No imperative for copula.
        ms: Sequence[VerbMood] = (VerbMood.Subj,)
        ss: Sequence[VPShape] = (
            VPShape.Declar,
            VPShape.Interrog,
            VPShape.RelDep,
            VPShape.RelIndep,
            VPShape.Report
        )
        ps: Sequence[VPPolarity] = (
            VPPolarity.Pos,
            VPPolarity.Neg
        )

        t: VerbTense
        s: VPShape
        p: VPPolarity
        m: VerbMood

        if tenses is None:
            tenses = {}
            for t in ts:
                tenses[t] = {}
                for s in ss:
                    tenses[t][s] = {}
                    for p in ps:
                        tenses[t][s][p] = []
        self.tenses = tenses

        if moods is None:
            moods = {}
            for m in ms:
                moods[m] = {}
                for p in ps:
                    moods[m][p] = []
        self.moods = moods
        # endregion

        self.disambig = disambig

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET._ElementTree]) -> "Copula":
        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml)

        root = doc.getroot()
        disambig = root.get("disambig", "")

        verb = cls(
            disambig=disambig,
        )
        tenses: TenseDictionary = verb.tenses
        moods: MoodDictionary = verb.moods

        # Helper methods to add forms quickly:
        def _addTense(
            t: VerbTense, d: VPShape, p: VPPolarity, form: str, pre_vowel: str | None
        ) -> None:
            tenses[t][d][p].append((Form(form), Form(pre_vowel) if pre_vowel is not None else None))

        def _addMood(m: VerbMood, p: VPPolarity, form: str, pre_vowel: str | None) -> None:
            moods[m][p].append((Form(form), Form(pre_vowel) if pre_vowel is not None else None))

        el: ET._Element
        value: str
        pre_vowel: str | None
        shape: VPShape
        polarity: VPPolarity
        tense: VerbTense

        for el in root.findall("./tenseForm"):
            value = el.get("default", "")
            pre_vowel = el.get("preVowel", None)
            tense = VerbTense(el.get("tense"))
            shape = VPShape(el.get("shape"))
            polarity = VPPolarity(el.get("polarity"))
            _addTense(tense, shape, polarity, value, pre_vowel)

        for el in root.findall("./moodForm"):
            value = el.get("default", "")
            pre_vowel = el.get("preVowel", None)
            mood = VerbMood(el.get("mood"))
            polarity = VPPolarity(el.get("polarity"))
            _addMood(mood, polarity, value, pre_vowel)

        return verb

    # Extracts the copula's lemma:
    def getLemma(self) -> str:
        return self.tenses[VerbTense.Pres][VPShape.Declar][VPPolarity.Pos][0][0].value

    # Prints the verb in BuNaMo format:
    def printXml(self) -> ET._ElementTree:
        root: ET._Element = ET.Element("copula")
        doc: ET._ElementTree = ET.ElementTree(root)

        root.set("default", self.getLemma())
        root.set("disambig", self.disambig)

        f: Form
        pre_vowel: Form | None 
        tense: VerbTense
        shape: VPShape
        polarity: VPPolarity

        for tense in self.tenses:
            for shape in self.tenses[tense]:
                for polarity in self.tenses[tense][shape]:
                    for f, pre_vowel in self.tenses[tense][shape][polarity]:
                        el = ET.SubElement(root, "tenseForm")
                        el.set("default", f.value)
                        if pre_vowel is not None:
                            el.set("preVowel", pre_vowel.value)
                        el.set("tense", tense.value)
                        el.set("shape", shape.value)
                        el.set("person", polarity.value)

        mood: VerbMood
        for mood in self.moods:
            for polarity in self.moods[mood]:
                for f, pre_vowel in self.moods[mood][polarity]:
                    el = ET.SubElement(root, "moodForm")
                    el.set("default", f.value)
                    if pre_vowel is not None:
                        el.set("preVowel", pre_vowel.value)
                    el.set("mood", mood.value)
                    el.set("polarity", polarity.value)

        return doc
    @property
    def tenses_flattened(self):
        for tense in self.tenses.values():
            for shape in tense.values():
                for polarity in shape.values():
                    for default, pre_vowel in polarity:
                        yield default
                        if pre_vowel is not None:
                            yield pre_vowel

    @property
    def moods_flattened(self):
        for mood in self.moods.values():
            for polarity in mood.values():
                for default, pre_vowel in polarity:
                    yield default
                    if pre_vowel is not None:
                        yield pre_vowel
