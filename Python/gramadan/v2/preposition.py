from typing import Optional, Union
from lxml import etree as ET

from gramadan import preposition
from .features import Form, FormList
from .entity import Entity
from .np import NP
from .noun import Noun
preposition.Form = Form
preposition.FormList = FormList

# A class for a preposition:
class Preposition(Entity[preposition.Preposition]):
    super_cls = preposition.Preposition

    _form_fields = (
        'sg1',
        'sg2',
        'sg3Masc',
        'sg3Fem',
        'pl1',
        'pl2',
        'pl3',
        'sgArt',
        'sgArtVowel',
        'sgPoss2',
        'sgPoss3Masc',
        'sgPoss3Fem',
        'plArt',
        'plPoss1',
        'plPoss2',
        'plPoss3',
        'copularCons',
        'copularPresVowel',
        'copularPastVowel',
    )

    def __init__(
        self,
        lemma: str = '',
        sg1: Optional[list[Form]] = None,
        sg2: Optional[list[Form]] = None,
        sg3Masc: Optional[list[Form]] = None,
        sg3Fem: Optional[list[Form]] = None,
        pl1: Optional[list[Form]] = None,
        pl2: Optional[list[Form]] = None,
        pl3: Optional[list[Form]] = None,
        disambig: str = "",
        v1: Optional[preposition.Preposition] = None,
        **kwargs
    ):
        super().__init__(
            lemma=lemma,
            sg1=sg1,
            sg2=sg2,
            sg3Masc=sg3Masc,
            sg3Fem=sg3Fem,
            pl1=pl1,
            pl2=pl2,
            pl3=pl3,
            disambig=disambig,
            v1=v1,
        )
        supplementary_fields = self._form_fields[8:]
        self._forms.update({
            field: FormList() if field not in kwargs else kwargs[field]
            for field in supplementary_fields
        })

    # Constructor:
    @classmethod
    def from_strings(
        cls,
        *args,
        **kwargs
    ) -> 'Preposition':
        v1 = preposition.Preposition.from_strings(*args, **kwargs)
        return cls(v1=v1)

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET._ElementTree], keep_doc: bool=False) -> Entity:
        if isinstance(doc, str):
            xml = ET.parse(doc)
            return cls.create_from_xml(xml, keep_doc=keep_doc)
        root = doc.getroot()
        lemma = root.get("default", "")
        disambig = root.get("disambig", "")

        el: ET._Element
        forms: dict[str, list[Form]] = {}

        for form in cls._form_fields:
            for el in root.findall(f"./{form}"):
                if form not in forms:
                    forms[form] = FormList()
                forms[form].append(Form(el.get("default", "")))

        v2 = cls(lemma, disambig=disambig, v1=None, **forms)
        if keep_doc:
            cls._definition = doc

        return v2

    # Prints the preposition in BuNaMo format:
    def printXml(self) -> ET._ElementTree:
        root: ET._Element = ET.Element("preposition")
        doc: ET._ElementTree = ET.ElementTree(root)
        f: Form

        root.set("default", self.lemma)
        root.set("disambig", self.disambig)

        for field, forms in self.forms.items():
            for f in forms:
                el = ET.SubElement(root, field)
                el.set("default", f.value)

        return doc

    def __add__(self, other: NP | Noun) -> Entity:
        from .pp import PP

        if isinstance(other, NP):
            ...
        elif isinstance(other, Noun):
            other = NP.create_from_noun(other)
        return PP.create(self, other)
