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
    RelDep = auto()
    RelIndep = auto()
    Report = auto()
    # , Report*/ }

class VerbDependency(AutoName):
    Indep = auto()
    Dep = auto()
    # PTW: Had to add here to make parseable
    # e.g. a bheas
    RelIndep = auto()

class VerbConjugationClass(AutoName):
    Irregular = auto()
    First = auto()
    Second = auto()

verb.Form = Form
verb.VPShapeType = VPShape
verb.VD = VerbDependency

IRREGULARS = {
    "abair",
    "beir",
    "bí",
    "cluin",
    "clois",
    "déan",
    "faigh",
    "feic",
    "ith",
    "tabhair",
    "tar",
    "téigh",
}

# A verb:
class Verb(Entity[verb.Verb]):
    super_cls = verb.Verb

    _form_fields = (
        'verbalNoun',
        'verbalAdjective',
        'tenses_flattened',
        'moods_flattened',
    )

    def get_conjugation(self) -> VerbConjugationClass:
        lemma = self.getLemma()

        if lemma in IRREGULARS:
            return VerbConjugationClass.Irregular

        # PTW: According to Gramadach na Gaeilge, the "real difference" between
        # the two conjugations, is the use of 'f' in the future, which is easy for us
        # to check. TODO: see if this is genuinely universal (seems so by the Caighdeán)

        # We have the root and the first person future - remove the overlapping start,
        # then we should be left with the future suffix and any mutations, the latter of which
        # should not include any 'f's (TODO: are there any exceptions?)
        future: str = self.tenses[verb.VerbTense.Fut][verb.VD.Indep][verb.VerbPerson.Base][0].value
        for n, (l, m) in enumerate(zip(future, lemma)):
            if l != m:
                break
        future_suffix = future[n:]
        if "f" in future_suffix:
            return VerbConjugationClass.First
        else:
            return VerbConjugationClass.Second

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

    def _populate_tense_rules(self) -> None:
        self._populate_original_tense_rules()
        Verb._populate_additional_tense_rules(self.tenseRules)

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET._ElementTree]) -> Verb:
        vrb: Verb = super().create_from_xml(doc)
        # This should be folded into _populate_tense_rules

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

        # PTW: Add in known relative dependent forms if missing
        # - these should really be in the original XMLs
        lemma = vrb.getLemma()
        # Except irregulars, to be safe
        if vrb.get_conjugation() != VerbConjugationClass.Irregular and lemma != "lean":
            for t in (verb.VerbTense.Pres, verb.VerbTense.PresCont, verb.VerbTense.Fut):
                if not (forms := vrb.tenses[t][verb.VD.RelIndep][verb.VerbPerson.Base]):
                    form_sg3 = vrb.tenses[t][verb.VD.Indep][verb.VerbPerson.Base]
                    if not form_sg3:
                        continue
                    form_sg3 = form_sg3[0].value
                    if t == verb.VerbTense.Fut:
                        # In this case, for this to work, we _must_ have an -idh in the
                        # third-person singular. By Wiktionary (and there are few online sources
                        # to go on), we expect dóigh -> dónn -> dós (ultimately, a dhós)
                        if not form_sg3.endswith("idh"):
                            raise RuntimeError(f"Could not work out missing relative independent form for {lemma}")
                        if form_sg3.endswith("fidh"):
                            form_relind = form_sg3[:-4] + "feas"
                        else:
                            # Strictly, Grammar na Gaeilge says -íos or -aíos but Wiktionary's
                            # got beannós (for example) and that turns up in An Tiomna Nua (1970)
                            form_relind = form_sg3[:-3] + "s"
                    else:
                        # In this case, for this to work, we _must_ have a double-n in the
                        # third-person singular. By Wiktionary (and there are few online sources
                        # to go on), we expect dóigh -> dónn -> dós (ultimately, a dhós)
                        # TODO: Double-check present-habitual has no surprises (outside irregulars)
                        if not form_sg3.endswith("nn"):
                            raise RuntimeError(f"Could not work out missing relative independent form for {lemma}")
                        form_relind = form_sg3[:-2] + "s"
                    forms.append(Form(form_relind))

        return vrb

    @staticmethod
    def _populate_additional_tense_rules(tense_rules) -> None:
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

        pols: Sequence[VPPolarity] = (VPPolarity.Pos, VPPolarity.Neg)
        t: VPTense
        p: VPPerson
        per: VPPerson
        s: VPShape
        pol: VPPolarity
        for t in ts:
            for per in pers:
                if per in (VPPerson.Sg3Masc, VPPerson.Sg3Fem):
                    ver_per = verb.VerbPerson.Sg3
                elif per == VPPerson.NoSubject:
                    ver_per = verb.VerbPerson.Base
                else:
                    ver_per = verb.VerbPerson[per.value]
                tense_rules[t][per][VPShape.RelIndep] = {pol: [] for pol in (VPPolarity.Pos, VPPolarity.Neg)}
                tense_rules[t][per][VPShape.RelDep] = {pol: [] for pol in (VPPolarity.Pos, VPPolarity.Neg)}
                for pol in pols:
                    rel_indep = tense_rules[t][per][VPShape.RelIndep][pol]
                    rel_dep = tense_rules[t][per][VPShape.RelDep][pol]
                    if t in (VPTense.PresCont, VPTense.Fut, VPTense.Past):
                        if pol == VPPolarity.Neg:
                            # ... nach cheapann
                            # ... nár cheap mé
                            # ... nach bhfaca mé
                            # This covers irregular behaviour, except faigh.
                            # TODO: Ensure this does not cover any incorrect forms.
                            # There is no independent negative.
                            rel_dep += copy.deepcopy(tense_rules[t][per][VPShape.Interrog][pol])
                        else:
                            tense = verb.VerbTense[t.value]
                            if t in (VPTense.PresCont, VPTense.Fut):
                                # ... a cheapann
                                # ... a fhaigheann
                                if per == VPPerson.NoSubject:
                                    # TODO: -as/-fas
                                    rel_indep.append(
                                        verb.VerbTenseRule("a", Mutation.Len1, tense, verb.VD.RelIndep, verb.VerbPerson.Base, "")
                                    )
                                    rel_indep.append(
                                        verb.VerbTenseRule("a", Mutation.Len1, tense, verb.VD.Indep, verb.VerbPerson.Base, "")
                                    )

                                rel_dep.append(
                                    verb.VerbTenseRule("a", Mutation.Ecl1, tense, verb.VD.Dep, ver_per, "")
                                )
                            elif t in (VPTense.Past,):
                                # ... a d'fhág
                                if per == VPPerson.NoSubject:
                                    rel_indep.append(
                                        verb.VerbTenseRule("a", Mutation.Len1, tense, verb.VD.Indep, verb.VerbPerson.Base, "")
                                    )

                                # ... ar cheap mé
                                base = tense_rules[t][per][VPShape.Interrog][pol][0]
                                particle = "a" if base.particle == "an" else base.particle
                                rel_dep.append(
                                    verb.VerbTenseRule(particle, base.mutation, tense, verb.VD.Dep, verb.VerbPerson.Base, "")
                                )
                    else:
                        # Conditional, PastCont, Autonomous
                        ...
        # Exceptions: faigh, bí, deir
        # Is fuair only an exception in Ulster (not bhfuair, like Caighdéan)?

# TODO: Hacky
verb.Verb._populate_original_tense_rules = verb.Verb._populate_tense_rules
verb.Verb._populate_tense_rules = Verb._populate_tense_rules
