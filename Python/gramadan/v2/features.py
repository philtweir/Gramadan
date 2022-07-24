from typing import Union, Optional
from dataclasses import dataclass
from gramadan.features import FormV1, Gender, Strength

class Form(FormV1):
    test = 0
    def __eq__(self, other):
        if isinstance(other, FormV1):
            return super().__eq__(other)
        return self.value == other


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
