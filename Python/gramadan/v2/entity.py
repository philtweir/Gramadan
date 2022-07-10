from lxml import etree as ET
from typing import Union, TypeVar, Generic, Type
from gramadan.verb import Verb

T = TypeVar('T')
class Entity(Generic[T]):
    v1 = None
    _forms: dict
    _form_fields: tuple = ()
    super_cls: Type[T]

    def __repr__(self):
        return self.getLemma()

    def __iter__(self):
        for field, forms in self.forms.items():
            yield field, forms

    def __getattr__(self, attr):
        return getattr(self.v1, attr)

    def __setattribute__(self, attr, val):
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
    def create_from_xml(cls, doc: Union[str, ET._ElementTree]) -> 'Entity':
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
