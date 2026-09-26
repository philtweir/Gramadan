from __future__ import annotations

from lxml import etree as ET
from typing import Optional, Union

from .features import Form, FormList

# A class for a preposition:
class Preposition:
    disambig: str = ""

    def getNickname(self) -> str:
        ret: str = self.lemma + " prep"
        if self.disambig != "":
            ret += " " + self.disambig
        ret = ret.replace(" ", "_")
        return ret

    def getLemma(self) -> str:
        return self.lemma

    def __init__(
        self,
        lemma: str,
        sg1: Optional[list[Form]],
        sg2: Optional[list[Form]],
        sg3Masc: Optional[list[Form]],
        sg3Fem: Optional[list[Form]],
        pl1: Optional[list[Form]],
        pl2: Optional[list[Form]],
        pl3: Optional[list[Form]],
        disambig: str = "",
    ):

        # The lemma:
        self.lemma: str = lemma

        self.disambig = disambig

        # Inflected forms (for number, person and gender):
        self.sg1: list[Form] = FormList()
        if sg1 is not None:
            self.sg1 = sg1

        self.sg2: list[Form] = FormList()
        if sg2 is not None:
            self.sg2 = sg2

        self.sg3Masc: list[Form] = FormList()
        if sg3Masc is not None:
            self.sg3Masc = sg3Masc

        self.sg3Fem: list[Form] = FormList()
        if sg3Fem is not None:
            self.sg3Fem = sg3Fem

        self.pl1: list[Form] = FormList()
        if pl1 is not None:
            self.pl1 = pl1

        self.pl2: list[Form] = FormList()
        if pl2 is not None:
            self.pl2 = pl2

        self.pl3: list[Form] = FormList()
        if pl3 is not None:
            self.pl3 = pl3

    # Returns true if the preposition has no infected forms:
    def isEmpty(self) -> bool:
        return (
            len(self.sg1)
            + len(self.sg2)
            + len(self.sg3Masc)
            + len(self.sg3Fem)
            + len(self.pl1)
            + len(self.pl2)
            + len(self.pl3)
            == 0
        )

    # Constructor:
    @classmethod
    def from_strings(
        cls,
        lemma: str,
        sg1Str: str,
        sg2Str: str,
        sg3MascStr: str,
        sg3FemStr: str,
        pl1Str: str,
        pl2Str: str,
        pl3Str: str,
    ) -> Preposition:
        sg1 = FormList()
        if sg1Str != "":
            sg1.append(Form(sg1Str))

        sg2 = FormList()
        if sg2Str != "":
            sg2.append(Form(sg2Str))

        sg3Masc = FormList()
        if sg3MascStr != "":
            sg3Masc.append(Form(sg3MascStr))

        sg3Fem = FormList()
        if sg3FemStr != "":
            sg3Fem.append(Form(sg3FemStr))

        pl1 = FormList()
        if pl1Str != "":
            pl1.append(Form(pl1Str))

        pl2 = FormList()
        if pl2Str != "":
            pl2.append(Form(pl2Str))

        pl3 = FormList()
        if pl3Str != "":
            pl3.append(Form(pl3Str))

        return cls(
            lemma,
            sg1=sg1,
            sg2=sg2,
            sg3Masc=sg3Masc,
            sg3Fem=sg3Fem,
            pl1=pl1,
            pl2=pl2,
            pl3=pl3,
        )

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET._ElementTree]) -> Preposition:
        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml)
        root = doc.getroot()
        lemma = root.get("default", "")
        disambig = root.get("disambig", "")

        sg1: list[Form] = FormList()
        sg2: list[Form] = FormList()
        sg3Masc: list[Form] = FormList()
        sg3Fem: list[Form] = FormList()
        pl1: list[Form] = FormList()
        pl2: list[Form] = FormList()
        pl3: list[Form] = FormList()

        el: ET._Element
        for el in root.findall("./sg1"):
            sg1.append(Form(el.get("default", "")))

        for el in root.findall("./sg2"):
            sg2.append(Form(el.get("default", "")))

        for el in root.findall("./sg3Masc"):
            sg3Masc.append(Form(el.get("default", "")))

        for el in root.findall("./sg3Fem"):
            sg3Fem.append(Form(el.get("default", "")))

        for el in root.findall("./pl1"):
            pl1.append(Form(el.get("default", "")))

        for el in root.findall("./pl2"):
            pl2.append(Form(el.get("default", "")))

        for el in root.findall("./pl3"):
            pl3.append(Form(el.get("default", "")))

        return cls(lemma, sg1, sg2, sg3Masc, sg3Fem, pl1, pl2, pl3, disambig)

    # Prints the preposition in BuNaMo format:
    def printXml(self) -> ET._ElementTree:
        root: ET._Element = ET.Element("preposition")
        doc: ET._ElementTree = ET.ElementTree(root)
        f: Form

        root.set("default", self.lemma)
        root.set("disambig", self.disambig)

        for f in self.sg1:
            el = ET.SubElement(root, "sg1")
            el.set("default", f.value)

        for f in self.sg2:
            el = ET.SubElement(root, "sg2")
            el.set("default", f.value)

        for f in self.sg3Masc:
            el = ET.SubElement(root, "sg3Masc")
            el.set("default", f.value)

        for f in self.sg3Fem:
            el = ET.SubElement(root, "sg3Fem")
            el.set("default", f.value)

        for f in self.pl1:
            el = ET.SubElement(root, "pl1")
            el.set("default", f.value)

        for f in self.pl2:
            el = ET.SubElement(root, "pl2")
            el.set("default", f.value)

        for f in self.pl3:
            el = ET.SubElement(root, "pl3")
            el.set("default", f.value)

        return doc
