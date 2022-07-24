from __future__ import annotations

from dataclasses import dataclass
from typing import Optional, Union, Sequence
from lxml import etree as ET
from gramadan import verb

from .features import Form, FormSg, Gender
from .entity import Entity
verb.Form = Form

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
