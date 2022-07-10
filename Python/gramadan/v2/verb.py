from __future__ import annotations

from dataclasses import dataclass
from typing import Optional, Union, Sequence
from lxml import etree as ET
from gramadan import verb

from .features import Form
from .entity import Entity
verb.Form = Form

# A verb:
class Verb(Entity[verb.Verb]):
    super_cls = verb.Verb

    _form_fields = (
        'verbalNoun',
        'verbalAdjective',
        'tenses_flattened',
        'moods_flattened',
    )

    @property
    def tenses_flattened(self):
        for tense in self.tenses.values():
            for dep in tense.values():
                for person in dep.values():
                    yield from person

    @property
    def moods_flattened(self):
        for mood in self.moods.values():
            for person in mood.values():
                yield from person
