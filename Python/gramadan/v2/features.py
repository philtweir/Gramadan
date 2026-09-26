from typing import Union, Optional, Any
from enum import auto
from dataclasses import dataclass
from collections import UserList

from gramadan.features import FormV1, Gender, Strength, AutoName
from .explainer import ExplainStr, finalize_form, Explainer, Explanation, Source, explaining

class FormList(UserList):
    name: str | None = None
    parent: Any | None = None

    def append(self, form):
        form._form_list = self
        return super().append(form)

class Form(FormV1):
    test = 0
    _value: ExplainStr
    _form_list: FormList | None

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        if args and not isinstance(args[0], ExplainStr):
            args = (ExplainStr(args[0]), *args[1:])
        super().__init__(*args, **kwargs)

    def __repr__(self) -> str:
        return self._value._str

    def __str__(self) -> str:
        return self._value._str

    @property
    def explanation(self) -> list[Explanation | str] | None:
        return self._value.explanation

    @property
    def value(self) -> ExplainStr:
        from_form = ExplainStr(self._value._str)
        if explaining():
            from_form.explanation = [Source(
                fm=self,
            )]
        return from_form

    @value.setter
    def value(self, value: ExplainStr | str) -> None:
        if not isinstance(value, ExplainStr):
            value = ExplainStr(value)
        self._value = value

    def __eq__(self, other):
        if isinstance(other, FormV1):
            return super().__eq__(other)
        return self.value == other


@dataclass
class FormVS(Form):
    pre_vowel_sandhi: Optional[str]

# Class for noun and noun phrase forms in the singular:
@dataclass
class FormSg(Form):
    gender: Optional[Gender]
    # We allow gender to be optional, as it may be unknown/undefined
    # (e.g. seemingly many verbal nouns and proper nouns) but use of this attribute should
    # then throw an error.


# Class for noun forms in the plural genitive:
@dataclass
class FormPlGen(Form):
    strength: Optional[Strength]
    # in the plural genitive, a noun form has strength.
    # We allow it to be optional, as it may be unknown/undefined
    # (e.g. some proper nouns) but use of this attribute should
    # then throw an error.

class Case(AutoName):
    Nom = auto()
    Gen = auto()
    Dat = auto()
    Voc = auto()

class System(AutoName):
    N = auto()
    S = auto()

class Article(AutoName):
    NoArt = auto()
    Art = auto()

class Number(AutoName):
    Sg = auto()
    Pl = auto()
