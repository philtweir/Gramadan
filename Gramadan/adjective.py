import xml.etree.ElementTree as ET
from typing import List, Optional, Union
from singular_info import SingularInfo
from features import Form


class Adjective:
    disambig: str = ""
    declension: int = 0
    isPre: bool = False

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET.ElementTree]):
        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml)

        root = doc.getroot()
        disambig = root.GetAttribute("disambig")

        kwargs = {}
        try:
            kwargs["declension"] = int(root.GetAttribute("declension"))
        except:
            pass
        try:
            kwargs["isPre"] = bool(root.GetAttribute("isPre"))
        except:
            pass

        sgNom: list[Form] = []
        el: XmlElement
        for el in doc.SelectNodes("/*/sgNom"):
            sgNom.Add(Form(el.GetAttribute("default")))

        sgGenMasc: list[Form] = []
        el: XmlElement
        for el in doc.SelectNodes("/*/sgGenMasc"):
            sgGenMasc.Add(Form(el.GetAttribute("default")))

        sgGenMasc: list[Form] = []
        el: XmlElement
        for el in doc.SelectNodes("/*/sgGenFem"):
            sgGenFem.Add(Form(el.GetAttribute("default")))

        sgVocMasc: list[Form] = []
        el: XmlElement
        for el in doc.SelectNodes("/*/sgVocMasc"):
            sgVocMasc.Add(Form(el.GetAttribute("default")))

        sgVocFem: list[Form] = []
        el: XmlElement
        for el in doc.SelectNodes("/*/sgVocFem"):
            sgVocFem.Add(Form(el.GetAttribute("default")))

        plNom: list[Form] = []
        el: XmlElement
        for el in doc.SelectNodes("/*/plNom"):
            plNom.Add(Form(el.GetAttribute("default")))

        graded: list[Form] = []
        el: XmlElement
        for el in doc.SelectNodes("/*/graded"):
            graded.Add(Form(el.GetAttribute("default")))

        abstractNoun: list[Form] = []
        el: XmlElement
        for el in doc.SelectNodes("/*/abstractNoun"):
            abstractNoun.Add(Form(el.GetAttribute("default")))

        return cls(
            sgNom=sgNom,
            sgGenMasc=sgGenMasc,
            sgGenFem=sgGenFem,
            sgVocMasc=sgVocMasc,
            sgVocFem=sgVocFem,
            plNom=plNom,
            graded=graded,
            abstractNoun=abstractNoun,
            disambig=disambig,
            **kwargs
        )

    @classmethod
    def create_from_singular_info(
        cls,
        sgMasc: SingularInfo,
        sgFem: SingularInfo,
        plural_or_graded: Optional[str],
        graded: Optional[str],
    ):
        if graded is None:
            plural = None
            graded = plural_or_graded
        else:
            plural = plural_or_graded
        return cls(
            sgNom=sgMasc.nominative,
            sgGenMasc=sgMasc.genitive,
            sgGenFem=sgFem.genitive,
            sgVocMasc=sgMasc.vocative,
            sgVocFem=sgFem.vocative,
            plural=None if plural is None else [Form(plural)],
            graded=[Form(graded)],
        )

    def __init__(
        self,
        sgNom: list[Form] = None,
        sgGenMasc: list[Form] = None,
        sgGenFem: list[Form] = None,
        sgVocMasc: list[Form] = None,
        sgVocFem: list[Form] = None,
        plNom: list[Form] = None,
        graded: list[Form] = None,
        abstractNoun: list[Form] = None,
        disambig="",
        declension: int = 0,
        isPre: bool = False,
    ):
        # The adjective's traditional declension class (not actually used for anything); default is 0 meaning none or unknown:

        # Forms of the adjective:
        self.sgNom: list[Form] = [] if sgNom is None else sgNom
        self.sgGenMasc: list[Form] = [] if sgGenMasc is None else sgGenMasc
        self.sgGenFem: list[Form] = [] if sgGenFem is None else sgGenFem
        self.sgVocMasc: list[Form] = [] if sgVocMasc is None else sgVocMasc
        self.sgVocFem: list[Form] = [] if sgVocFem is None else sgVocFem

        self.disambig = disambig
        self.declension = declension
        self.isPre = isPre

        # Adjective forms in the plural:
        self.plNom: list[Form] = [] if plNom is None else plNom

        # Form for grading:
        self.graded: list[Form] = [] if graded is None else graded

        # Related abstract noun:
        self.abstractNoun: list[Form] = [] if abstractNoun is None else abstractNoun

        # Whether the adjective is a prefix (like "sean"):

    # Returns the adjective's lemma:
    def getLemma() -> str:
        ret: str = ""
        lemmaForm: Form = self.sgNom[0]
        if lemmaForm is not None:
            ret = lemmaForm.value

        return ret

    # These return graded forms of the adjective:
    def getComparPres() -> list[Form]:  # comparative present, eg. "níos mó"
        ret: list[Form] = []
        gradedForm: Form
        for gradedForm in graded:
            ret.Add(Form("níos " + gradedForm.value))
        return ret

    def getSuperPres() -> list[Form]:  # superlative present, eg. "is mó"
        ret: list[Form] = []
        gradedForm: Form
        for gradedForm in graded:
            ret.Add(Form("is " + gradedForm.value))
        return ret

    def getComparPast() -> list[Form]:  # comparative past/conditional, eg. "ní ba mhó"
        ret: list[Form] = []
        gradedForm: Form
        for gradedForm in graded:
            form: str = ""
            if Regex.IsMatch(gradedForm.value, "^[aeiouáéíóúAEIOUÁÉÍÓÚ]"):
                form = "ní b'" + gradedForm.value
            elif Regex.IsMatch(gradedForm.value, "^f[aeiouáéíóúAEIOUÁÉÍÓÚ]"):
                form = "ní b'" + Opers.Mutate(Mutation.Len1, gradedForm.value)
            else:
                form = "ní ba " + Opers.Mutate(Mutation.Len1, gradedForm.value)
            ret.Add(Form(form))
        return ret

    def getSuperPast() -> list[Form]:  # superlative past/conditional, eg. "ba mhó"
        ret: list[Form] = []
        gradedForm: Form
        for gradedForm in graded:
            form: str = ""
            if Regex.IsMatch(gradedForm.value, "^[aeiouáéíóúAEIOUÁÉÍÓÚ]"):
                form = "ab " + gradedForm.value
            elif Regex.IsMatch(gradedForm.value, "^f"):
                form = "ab " + Opers.Mutate(Mutation.Len1, gradedForm.value)
            else:
                form = "ba " + Opers.Mutate(Mutation.Len1, gradedForm.value)
            ret.Add(Form(form))
        return ret

    def getNickname() -> str:
        ret: str = getLemma()
        ret += " adj"
        ret += self.declension.ToString() if self.declension > 0 else ""
        if self.disambig != "":
            ret += " " + self.disambig
        ret = ret.Replace(" ", "_")
        return ret

    def getFriendlyNickname() -> str:
        ret: str = getLemma()
        ret += " (adj"
        ret += self.declension.ToString() if self.declension > 0 else ""
        if self.disambig != "":
            ret += " " + self.disambig
        ret += ")"
        return ret

    # Prints the adjective in BuNaMo format:
    def printXml() -> ET.ElementTree:
        root: ET.Element = ET.Element("adjective")
        doc: ET.ElementTree = ET.ElementTree(root)
        root.set("default", self.getLemma())
        root.set("declension", self.declension.ToString())
        root.set("disambig", self.disambig)
        root.set("isPre", self.isPre.ToString())

        f: Form
        for f in self.sgNom:
            el: ET.Element = ET.SubElement(root, "sgNom")
            el.set("default", f.value)

        f: Form
        for f in self.sgGenMasc:
            el: ET.Element = ET.SubElement(root, "sgGenMasc")
            el.set("default", f.value)

        f: Form
        for f in self.sgGenFem:
            el: ET.Element = ET.SubElement(root, "sgGenFem")
            el.set("default", f.value)

        f: Form
        for f in self.sgVocMasc:
            el: ET.Element = ET.SubElement(root, "sgVocMasc")
            el.set("default", f.value)

        f: Form
        for f in self.sgVocFem:
            el: ET.Element = ET.SubElement(root, "sgVocFem")
            el.set("default", f.value)

        f: Form
        for f in self.plNom:
            el: ET.Element = ET.SubElement(root, "plNom")
            el.set("default", f.value)

        f: Form
        for f in self.graded:
            el: ET.Element = ET.SubElement(root, "graded")
            el.set("default", f.value)

        f: Form
        for f in self.abstractNoun:
            el: ET.Element = ET.SubElement(root, "abstractNoun")
            el.set("default", f.value)

        return doc
