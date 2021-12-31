import xml.etree.ElementTree as ET
from typing import Optional, Union

from features import Form

# A class for a preposition:
class Preposition:
    disambig: str = ""

    def getNickname(self) -> str:
        ret: str = this.lemma + " prep"
        if this.disambig != "":
            ret += " " + this.disambig
        ret = ret.Replace(" ", "_")
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
        self.sg1: list[Form] = []
        if sg1 is not None:
            self.sg1 = sg1

        self.sg2: list[Form] = []
        if sg2 is not None:
            self.sg2 = sg2

        self.sg3Masc: list[Form] = []
        if sg3Masc is not None:
            self.sg3Masc = sg3Masc

        self.sg3Fem: list[Form] = []
        if sg3Fem is not None:
            self.sg3Fem = sg3Fem

        self.pl1: list[Form] = []
        if pl1 is not None:
            self.pl1 = pl1

        self.pl2: list[Form] = []
        if pl2 is not None:
            self.pl2 = pl2

        self.pl3: list[Form] = []
        if pl3 is not None:
            self.pl3 = pl3

    # Returns true if the preposition has no infected forms:
    def isEmpty() -> bool:
        return (
            self.sg1.Count
            + self.sg2.Count
            + self.sg3Masc.Count
            + self.sg3Fem.Count
            + self.pl1.Count
            + self.pl2.Count
            + self.pl3.Count
            == 0
        )

    # Constructor:
    @classmethod
    def from_strings(
        cls,
        lemma: str,
        sg1: str,
        sg2: str,
        sg3Masc: str,
        sg3Fem: str,
        pl1: str,
        pl2: str,
        pl3: str,
    ):
        kwargs = {}
        if sg1 != "":
            kwargs["sg1"] = [Form(sg1)]
        if sg2 != "":
            kwargs["sg2"] = [Form(sg2)]
        if sg3Masc != "":
            kwargs["sg3Masc"] = [Form(sg3Masc)]
        if sg3Fem != "":
            kwargs["sg3Fem"] = [Form(sg3Fem)]
        if pl1 != "":
            kwargs["pl1"] = [Form(pl1)]
        if pl2 != "":
            kwargs["pl2"] = [Form(pl2)]
        if pl3 != "":
            kwargs["pl3"] = [Form(pl3)]
        cls(lemma, **kwargs)

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET.ElementTree]):
        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml)
        root = doc.getroot()
        lemma = root.get("default")
        disambig = root.get("disambig")

        sg1: list[Form] = []
        sg2: list[Form] = []
        sg3Masc: list[Form] = []
        sg3Fem: list[Form] = []
        pl1: list[Form] = []
        pl2: list[Form] = []
        pl3: list[Form] = []

        el: ET.Element
        for el in doc.SelectNodes("/*/sg1"):
            sg1.append(Form(el.get("default")))
        el: ET.Element
        for el in doc.SelectNodes("/*/sg2"):
            sg2.append(Form(el.get("default")))
        el: ET.Element
        for el in doc.SelectNodes("/*/sg3Masc"):
            sg3Masc.append(Form(el.get("default")))
        el: ET.Element
        for el in doc.SelectNodes("/*/sg3Fem"):
            sg3Fem.append(Form(el.get("default")))
        el: ET.Element
        for el in doc.SelectNodes("/*/pl1"):
            pl1.append(Form(el.get("default")))
        el: ET.Element
        for el in doc.SelectNodes("/*/pl2"):
            pl2.append(Form(el.get("default")))
        el: ET.Element
        for el in doc.SelectNodes("/*/pl3"):
            pl3.append(Form(el.get("default")))

        return cls(lemma, sg1, sg2, sg3Masc, sg3Fem, pl1, pl2, pl3, disambig)

    # Prints the preposition in BuNaMo format:
    def printXml(self) -> ET.ElementTree:
        root: ET.Element = ET.Element("preposition")
        doc: ET.ElementTree = ET.ElementTree(root)

        root.set("default", self.lemma)
        root.set("disambig", self.disambig)
        f: Form
        for f in sg1:
            el = ET.SubElement(root, "sg1")
            el.set("default", f.value)

        f: Form
        for f in sg2:
            el = ET.SubElement(root, "sg2")
            el.set("default", f.value)

        f: Form
        for f in sg3Masc:
            el = ET.SubElement(root, "sg3Masc")
            el.set("default", f.value)

        f: Form
        for f in sg3Fem:
            el = ET.SubElement(root, "sg3Fem")
            el.set("default", f.value)

        f: Form
        for f in pl1:
            el = ET.SubElement(root, "pl1")
            el.set("default", f.value)

        f: Form
        for f in pl2:
            el = ET.SubElement(root, "pl2")
            el.set("default", f.value)

        f: Form
        for f in pl3:
            el = ET.SubElement(root, "pl3")
            el.set("default", f.value)

        return doc
