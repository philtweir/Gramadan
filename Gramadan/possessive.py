import xml.etree.ElementTree as ET
from typing import Optional, Union
from features import Mutation, Form

# A possessive pronoun:
class Possessive:
    disambig: str = ""

    def getNickname() -> str:
        ret: str = getLemma()
        if self.disambig != "":
            ret += " " + self.disambig
        ret += " poss"
        ret = ret.Replace(" ", "_")
        return ret

    def getFriendlyNickname() -> str:
        ret: str = getLemma()
        if self.disambig != "":
            ret += " (" + self.disambig + ")"
        return ret

    def __init__(
        self,
        full: Optional[list[Form]],
        apos: Optional[list[Form]],
        mutation: Mutation = Mutation.Nil,
    ):
        # Its forms:
        self.full: list[Form] = []
        if full:
            self.full = full
        self.apos: list[Form] = []
        if apos:
            self.apos = apos

        # The mutation it causes:
        self.mutation = mutation

    # Returns the noun's lemma:
    def getLemma() -> str:
        ret: str = ""
        lemmaForm: Form = self.full[0]
        if lemmaForm != null:
            ret = lemmaForm.value
        return ret

    # Constructors:
    @classmethod
    def create_from_string_and_mutation(cls, s: str, mutation: Mutation):
        return cls(full=[Form(s)], apos=[Form(s)], mutation=mutation)

    @classmethod
    def create_from_strings_and_mutation(cls, full: str, apos: str, mutation: Mutation):
        return cls(full=[Form(full)], apos=[Form(apos)], mutation=mutation)

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET.ElementTree]):
        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml)

        root = doc.getroot()
        disambig = root.get("disambig")
        mutation = Mutation.Parse(
            typeof(Mutation), Utils.UpperInit(root.get("mutation"))
        )
        full: list[Form] = []
        apos: list[Form] = []

        el: ET.Element
        for el in doc.SelectNodes("/*/full"):
            full.append(Form(el.get("default")))

        el: ET.Element
        for el in doc.SelectNodes("/*/apos"):
            apos.append(Form(el.get("default")))

        return cls(full=full, apos=apos, mutation=mutation, disambig=disambig)

    # Prints the possessive pronoun in BuNaMo format:
    def printXml() -> ET.ElementTree:
        root: ET.Element = ET.Element("possessive")
        doc: ET.ElementTree = ET.ElementTree(root)
        root.set("default", self.getLemma())
        root.set("disambig", self.disambig)
        root.set("mutation", Utils.LowerInit(self.mutation.ToString()))

        f: Form
        for f in self.full:
            el = ET.SubElement(root, "full")
            el.set("default", f.value)

        f: Form
        for f in self.apos:
            el = ET.SubElement(root, "apos")
            el.set("default", f.value)
        return doc
