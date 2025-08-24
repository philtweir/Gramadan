from __future__ import annotations

import copy
from dataclasses import dataclass
from typing import Optional, Union, Sequence
from lxml import etree as ET
from gramadan import verb
from gramadan.features import Mutation
from gramadan.verb import VPPerson, VPPolarity, VPTense

from .features import Form, FormSg, Gender, AutoName, auto
from .entity import Entity

class VPShape(AutoName):
    Any = auto()
    Declar = auto()
    Interrog = auto()
    RelDepDir = auto()
    RelDepIndir = auto()
    Report = auto()
    # , RelIndep, Report*/ }

verb.Form = Form
verb.VPShapeType = VPShape

# A verb:
class Verb(Entity[verb.Verb]):
    super_cls = verb.Verb

    _form_fields = (
        'verbalNoun',
        'verbalAdjective',
        'tenses_flattened',
        'moods_flattened',
    )

    @property
    def tenses_flattened(self):
        for tense in self.tenses.values():
            for dep in tense.values():
                for person in dep.values():
                    yield from person

    @property
    def moods_flattened(self):
        for mood in self.moods.values():
            for person in mood.values():
                yield from person

    @property
    def gender(self):
        # While a verb does not have a gender,
        # verbal nouns do (this allows us to
        # represent a verbal noun as a specific form
        # of the verb, rather than creating a separate
        # noun object and losing the relationship to
        # the rest of the verb forms)
        if self.verbalNoun:
            return self.verbalNoun[0].gender

    def printXml(self) -> ET._ElementTree:
        doc = self.v1.printXml()
        root = doc.getroot()

        for el in root.findall("./verbalNoun"):
            root.remove(el)

        for f in self.verbalNoun:
            el = ET.SubElement(root, "verbalNoun")
            el.set("default", f.value)
            el.set("gender", f.gender)

        doc = ET.ElementTree(root)

        return doc

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET._ElementTree]) -> Verb:
        vrb: Verb = super().create_from_xml(doc)
        # This should be folded into _populate_tense_rules
        vrb._populate_additional_tense_rules()

        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml)

        root = doc.getroot()
        verbalNoun: list[Form] = []
        for el in root.findall("./verbalNoun"):
            verbalNoun.append(
                FormSg(
                    value=el.get("default", ""),
                    gender=Gender.Fem if el.get("gender") == "fem" else \
                        Gender.Masc if el.get("gender") == "masc" else None
                )
            )
        vrb._forms["verbalNoun"] = verbalNoun

        return vrb

    def _populate_additional_tense_rules(self) -> None:
        ts: Sequence[VPTense] = (
            VPTense.Past,
            VPTense.PastCont,
            VPTense.Pres,
            VPTense.PresCont,
            VPTense.Fut,
            VPTense.Cond,
        )
        pers: Sequence[VPPerson] = (
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

        tense_rules = self.tenseRules

        pols: Sequence[VPPolarity] = (VPPolarity.Pos, VPPolarity.Neg)
        t: VPTense
        p: VPPerson
        per: VPPerson
        s: VPShapeType
        pol: VPPolarity
        for t in ts:
            for per in pers:
                tense_rules[t][per][VPShape.RelDepDir] = {pol: [] for pol in (VPPolarity.Pos, VPPolarity.Neg)}
                tense_rules[t][per][VPShape.RelDepIndir] = {pol: [] for pol in (VPPolarity.Pos, VPPolarity.Neg)}
                for pol in pols:
                    direct = tense_rules[t][per][VPShape.RelDepDir][pol]
                    indirect = tense_rules[t][per][VPShape.RelDepIndir][pol]
                    if t in (VPTense.PresCont, VPTense.Fut, VPTense.Past):
                        if pol == VPPolarity.Neg:
                            # ... nach cheapaim
                            # ... nár cheap mé
                            # ... nach bhfaca mé
                            # This covers irregular behaviour, except faigh.
                            # TODO: Ensure this does not cover any incorrect forms.
                            direct += copy.deepcopy(tense_rules[t][per][VPShape.Interrog][pol])
                            indirect += copy.deepcopy(tense_rules[t][per][VPShape.Interrog][pol])
                        else:
                            tense = verb.VerbTense[t.value]
                            if t in (VPTense.PresCont, VPTense.Fut):
                                # ... a cheapaim
                                # ... a fhaigheann
                                direct.append(
                                    verb.VerbTenseRule("a", Mutation.Len1, tense, verb.VerbDependency.Indep, verb.VerbPerson.Base, "")
                                )
                                # TODO: -as/-fas

                                indirect.append(
                                    verb.VerbTenseRule("a", Mutation.Ecl1, tense, verb.VerbDependency.Dep, verb.VerbPerson.Base, "")
                                )
                            elif t in (VPTense.Past,):
                                # ... ar cheap mé
                                direct.append(
                                    verb.VerbTenseRule("a", Mutation.Len1, tense, verb.VerbDependency.Indep, verb.VerbPerson.Base, "")
                                )

                                base = tense_rules[t][per][VPShape.Interrog][pol][0]
                                particle = "a" if base.particle == "an" else base.particle
                                indirect.append(
                                    verb.VerbTenseRule(particle, base.mutation, tense, verb.VerbDependency.Dep, verb.VerbPerson.Base, "")
                                )
                    else:
                        # Conditional, PastCont, Autonomous
                        ...
        # Exceptions: faigh, bí, deir
        # Is fuair only an exception in Ulster (not bhfuair, like Caighdéan)?
