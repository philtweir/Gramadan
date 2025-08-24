from __future__ import annotations

from lxml import etree as ET
from typing import Union, TypeVar, Generic, Type
from gramadan.verb import Verb
from gramadan.features import Number, AutoName
from gramadan.v2.features import Case, Gender, System, Article

T = TypeVar('T')
class Entity(Generic[T]):
    v1: T
    _forms: dict = {} # Prevents __getattr__ being called
    _form_fields: tuple = ()
    super_cls: Type[T]

    def __repr__(self):
        return self.getLemma()

    def __iter__(self):
        for field, forms in self.forms.items():
            yield field, forms

    def __getattr__(self, attr):
        if attr == "v1": # This allows us to copy
            raise AttributeError()

        if attr in self._forms:
            return self._forms[attr]
        return getattr(self.v1, attr)

    def __setattribute__(self, attr, val):
        if attr in self._forms:
            self._forms[attr] = val
        return setattr(self.v1, attr, val)

    # Forms of the preposition:
    def __init__(
        self,
        *args,
        v1: T = None,
        **kwargs
    ):
        self._forms = {}
        if v1:
            self.v1 = v1
        else:
            self.v1 = self.super_cls(*args, **kwargs)

    @property
    def forms(self) -> dict[str, list]:
        forms = {}
        for form in self._form_fields:
            forms[form] = list(getattr(self, form, []))
        forms.update(self._forms)
        return forms

    @classmethod
    def create_from_xml(cls, doc: Union[str, ET._ElementTree]):
        v1 = cls.super_cls.create_from_xml(doc)
        return cls(v1=v1)

    def search(self, key, field):
        if field:
            if field not in self._form_fields:
                raise KeyError(field)
            return key in getattr(self, field)

        for field, forms in self.forms.items():
            if key in forms:
                return True
        return False

    def to(self, *features: list[AutoName]):
        gender = [gdr for gdr in features if isinstance(gdr, Gender)]
        number = [num for num in features if isinstance(num, Number)]
        case = [cas for cas in features if isinstance(cas, Case)]
        system = [sys for sys in features if isinstance(sys, System)]
        article = [art for art in features if isinstance(art, Article)]

        if any(len(descriptor) > 2 for descriptor in (gender, number, case)):
            raise RuntimeError("Cannot specify more than one descriptor at a time")

        form_name: str = ""
        if number:
            form_name += "Pl" if number[0] == Number.Pl else "Sg"
        if case:
            case = case[0]
            if case == Case.Gen:
                form_name += "Gen"
            elif case == Case.Voc:
                form_name += "Voc"
            elif case == Case.Nom:
                form_name += "Nom"
            elif case == Case.Dat:
                form_name += "Dat"
        if article and article[0] == Article.Art:
            form_name += "Art"
        if gender:
            if gender[0] == Gender.Masc:
                form_name += "Masc"
            elif gender[0] == Gender.Fem:
                form_name += "Fem"
        if system:
            if system[0] == System.N:
                form_name += "N"
            elif gender[0] == System.S:
                form_name += "S"
        form_name = form_name[0].lower() + form_name[1:]
        return self.forms[form_name]
