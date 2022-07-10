from typing import Union
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
    gender: Gender


# Class for noun forms in the plural genitive:
@dataclass
class FormPlGen(Form):
    strength: Strength
    # in the plural genitive, a noun form has strength.
