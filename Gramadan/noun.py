import xml.etree.ElementTree as ET
from typing import Optional, Union

from features import FormSg, Form, FormPlGen, Gender
from singular_info import SingularInfo
from plural_info import PluralInfo

# A noun
class Noun:
    disambig: str = ""

    def getNickname() -> str:
        ret: str = getLemma()
        ret += " masc" if this.getGender() == Gender.Masc else " fem"
        ret += this.declension.ToString() if this.declension > 0 else ""
        if this.disambig != "":
            ret += " " + this.disambig
        ret = ret.Replace(" ", "_")
        return ret

    def getFriendlyNickname() -> str:
        ret: str = getLemma()
        ret += " ("
        ret += "masc" if this.getGender() == Gender.Masc else "fem"
        ret += this.declension.ToString() if this.declension > 0 else ""
        if this.disambig != "":
            ret += " " + this.disambig
        ret += ")"
        return ret

    # The noun's traditional declension class (not actually used for anything); default is 0 meaning "none":
    declension: int = 0

    isProper: bool = False
    # If true, then all article-less genitives are always lenited, no matter what.

    isImmutable: bool = False
    # Eg. "blitz", genitive singular "an blitz"

    # Whether this noun is already definite, even without an article:
    isDefinite: bool = False
    # If true, then no articled forms will be generated when this noun is converted into a noun phrase.

    # For definite noun (isDefinite==true), whether an articled genitive may be generated
    allowArticledGenitive: bool = False
    # Eg. "na hÉireann", "na Gaillimhe"

    def __init__(
        self,
        sgNom: Optional[list[FormSg]] = None,
        sgGen: Optional[list[FormSg]] = None,
        sgVoc: Optional[list[FormSg]] = None,
        sgDat: Optional[list[FormSg]] = None,
        plNom: Optional[list[Form]] = None,
        plGen: Optional[list[FormPlGen]] = None,
        plVoc: Optional[list[Form]] = None,
        count: Optional[list[Form]] = None,
        isProper: bool = False,
        isImmutable: bool = False,
        isDefinite: bool = False,
        allowArticledGenitive: bool = False,
        disambig: str = ""
    ):
        # Noun forms in the singular:
        self.sgNom: list[FormSg] = [] if sgNom is None else sgNom
        self.sgGen: list[FormSg] = [] if sgNom is None else sgNom
        self.sgVoc: list[FormSg] = [] if sgNom is None else sgNom
        self.sgDat: list[FormSg] = [] if sgNom is None else sgNom

        # Noun forms in the plural:
        self.plNom: list[Form] = [] if sgNom is None else sgNom
        self.plGen: list[FormPlGen] = [] if sgNom is None else sgNom
        self.plVoc: list[Form] = [] if sgNom is None else sgNom

        # Noun form for counting (if any):
        self.count: list[Form] = [] if sgNom is None else sgNom

        # Whether this is a proper name:
        self.isProper = isProper

        # Whether this noun cannot be mutated (overrides isProper):
        isImmutable = isImmutable
        # Eg. "blitz", genitive singular "an blitz"

        # Whether this noun is already definite, even without an article:
        isDefinite = isDefinite
        # If true, then no articled forms will be generated when this noun is converted into a noun phrase.

        # For definite noun (isDefinite==true), whether an articled genitive may be generated
        allowArticledGenitive = allowArticledGenitive
        # Eg. "na hÉireann", "na Gaillimhe"

        self.disambig = disambig

        self._AddDative()

    # Returns the noun's lemma:
    def getLemma() -> str:
        ret: str = ""
        lemmaForm: Form = this.sgNom[0]
        if lemmaForm is not None:
            ret = lemmaForm.value
        return ret

    # Returns the noun's gender:
    def getGender() -> Gender:
        return this.sgNom[0].gender

    # Constructors:
    @classmethod
    def create_from_info(cls, si: SingularInfo, pi: Optional[PluralInfo]):
        sgNom: list[FormSg] = []
        wf: Form
        for wf in si.nominative:
            sgNom.append(FormSg(wf.value, si.gender))

        sgGen: list[FormSg] = []
        wf: Form
        for wf in si.genitive:
            sgGen.append(FormSg(wf.value, si.gender))

        sgVoc: list[FormSg] = []
        wf: Form
        for wf in si.vocative:
            sgVoc.append(FormSg(wf.value, si.gender))

        sgDat: list[FormSg] = []
        wf: Form
        for wf in si.dative:
            sgDat.append(FormSg(wf.value, si.gender))

        plNom: list[Form] = []
        plGen: list[FormPlGen] = []
        plVoc: list[Form] = []
        if pi is not None:
            wf: Form
            for wf in pi.nominative:
                plNom.append(Form(wf.value))

            wf: Form
            for wf in pi.genitive:
                plGen.append(FormPlGen(wf.value, pi.strength))

            wf: Form
            for wf in pi.vocative:
                plVoc.append(Form(wf.value))

        obj = cls(
            sgNom=sgNom,
            sgGen=sgGen,
            sgVoc=sgVoc,
            sgDat=sgDat,
            plNom=plNom,
            plGen=plGen,
            plVoc=plVoc,
        )
        return obj

    @classmethod
    def from_str(
        cls,
        gender: Gender,
        sgNom: str,
        sgGen: str,
        sgVoc: str,
        strength: str,
        plNom: str,
        plGen: str,
        plVoc: str,
    ):
        obj = cls(
            sgNom=[FormSg(sgNom, gender)],
            sgGen=[FormSg(sgGen, gender)],
            sgVoc=[FormSg(sgVoc, gender)],
            sgDat=[FormSg(sgNom, gender)],
            plNom=[Form(plNom)],
            plGen=[FormPlGen(plGen, strength)],
            plVoc=[Form(plVoc)],
        )
        return obj

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET.ElementTree]):
        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml)

        root = doc.getroot()
        disambig = root.get("disambig")

        kwargs = {}
        try:
            kwargs["declension"] = int(root.get("declension"))
        except:
            pass

        isProper = root.get("isProper") == "1"
        isImmutable = root.get("isImmutable") == "1"
        isDefinite = root.get("isDefinite") == "1"
        allowArticledGenitive = root.get("allowArticledGenitive") == "1"

        sgNom: list[FormSg] = []
        el: ET.Element
        for el in doc.SelectNodes("/*/sgNom"):
            sgNom.append(
                FormSg(
                    el.get("default"),
                    Gender.Fem if el.get("gender") == "fem" else Gender.Masc,
                )
            )

        sgNom: list[FormSg] = []
        el: ET.Element
        for el in doc.SelectNodes("/*/sgGen"):
            sgGen.append(
                FormSg(
                    el.get("default"),
                    Gender.Fem if el.get("gender") == "fem" else Gender.Masc,
                )
            )

        sgVoc: list[FormSg] = []
        el: ET.Element
        for el in doc.SelectNodes("/*/sgVoc"):
            sgVoc.append(
                FormSg(
                    el.get("default"),
                    Gender.Fem if el.get("gender") == "fem" else Gender.Masc,
                )
            )

        sgDat: list[FormSg] = []
        el: ET.Element
        for el in doc.SelectNodes("/*/sgDat"):
            sgDat.append(
                FormSg(
                    el.get("default"),
                    Gender.Fem if el.get("gender") == "fem" else Gender.Masc,
                )
            )

        plNom: list[Form] = []
        el: ET.Element
        for el in doc.SelectNodes("/*/plNom"):
            plNom.append(Form(el.get("default")))

        plGen: list[FormPlGen] = []
        el: ET.Element
        for el in doc.SelectNodes("/*/plGen"):
            plGen.append(
                FormPlGen(
                    el.get("default"),
                    Strength.Weak if el.get("strength") == "weak" else Strength.Strong,
                )
            )

        plVoc: list[Form] = []
        el: ET.Element
        for el in doc.SelectNodes("/*/plVoc"):
            plVoc.append(Form(el.get("default")))

        count: list[Form] = []
        el: ET.Element
        for el in doc.SelectNodes("/*/count"):
            count.append(Form(el.get("default")))

        obj = cls(
            sgNom=sgNom,
            sgGen=sgGen,
            sgVoc=sgVoc,
            sgDat=sgNom,
            plNom=plNom,
            plGen=plGen,
            plVoc=plVoc,
            count=count,
            disambig=disambig,
            isProper=isProper,
            isImmutable=isImmutable,
            isDefinite=isDefinite,
            allowArticledGenitive=allowArticledGenitive,
        )
        return obj

    # Called from each constructor to make sure the noun has a dative singular:
    def _AddDative(self):
        if n.sgDat.Count == 0:
            f: FormSg
            for f in n.sgNom:
                n.sgDat.append(FormSg(f.value, f.gender))

    # Prints a user-friendly summary of the noun's forms:
    def print():
        ret: str = ""
        ret += "sgNom: "
        f: Form
        for f in this.sgNom:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "sgGen: "
        f: Form
        for f in this.sgGen:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "sgVoc: "
        f: Form
        for f in this.sgVoc:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "sgDat: "
        f: Form
        for f in this.sgDat:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plNom: "
        f: Form
        for f in this.plNom:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plGen: "
        f: Form
        for f in this.plGen:
            ret += "[" + f.value + "] "
        ret += "\n"
        ret += "plVoc: "
        f: Form
        for f in this.plVoc:
            ret += "[" + f.value + "] "
        ret += "\n"
        return ret

    # Prints the noun in BuNaMo format:
    def printXml(self) -> ET.ElementTree:
        root: ET.Element = ET.Element("noun")
        doc: ET.ElementTree = ET.ElementTree(root)

        root.set("default", self.getLemma())
        root.set("declension", self.declension.ToString())
        root.set("disambig", self.disambig)
        root.set("isProper", ("1" if self.isProper else "0"))
        root.set("isImmutable", ("1" if self.isImmutable else "0"))
        root.set("isDefinite", ("1" if self.isDefinite else "0"))
        root.set("allowArticledGenitive", ("1" if self.allowArticledGenitive else "0"))

        for f in self.sgNom:
            el: ET.Element = ET.SubElement(root, "sgNom")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        for f in self.sgGen:
            el: ET.Element = ET.SubElement(root, "sgGen")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        for f in self.sgVoc:
            el: ET.Element = ET.SubElement(root, "sgVoc")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        for f in self.sgDat:
            el: ET.Element = ET.SubElement(root, "sgDat")
            el.set("default", f.value)
            el.set("gender", ("masc" if f.gender == Gender.Masc else "fem"))

        for f in self.plNom:
            el: ET.Element = ET.SubElement(root, "plNom")
            el.set("default", f.value)

        for f in self.plGen:
            el: ET.Element = ET.SubElement(root, "plGen")
            el.set("default", f.value)
            el.set("strength", ("strong" if f.strength == Strength.Strong else "weak"))

        for f in self.plVoc:
            el: ET.Element = ET.SubElement(root, "plVoc")
            el.set("default", f.value)

        for f in self.count:
            el: ET.Element = ET.SubElement(root, "count")
            el.set("default", f.value)

        return doc
