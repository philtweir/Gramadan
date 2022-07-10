import sys
import os
from lxml import etree as ET
from typing import Optional, Union, TypedDict, Iterable, Type, TypeVar, TYPE_CHECKING, Callable, Generator
from collections import UserDict
from gramadan.opers import Opers
from .np import NP
from .entity import Entity as EntityType
from .verb import Verb
from .noun import Noun
from .adjective import Adjective
from .preposition import Preposition

# EntityType = Union[
#     Noun,
#     Adjective,
#     Preposition,
#     NP,
#     Verb
# ]

UPOS_TYPE_MAP = {
    "NOUN": "noun", "ADJ": "adjective", "ADP": "preposition",
    "VERB": "verb"
}

ENTITY_TYPE_MAP = {
    "noun": Noun, "adjective": Adjective, "preposition": Preposition,
    "nounPhrase": NP, "verb": Verb
}

S = TypeVar('S')
T = TypeVar('T', bound=EntityType)
if TYPE_CHECKING:
    UserDictType = UserDict[S, T]
else:
    UserDictType = UserDict

class DeepsearchingDict(UserDictType[S, T]):
    def search(self, term: str, field: Optional[str]):
        for k, v in self.data.items():
            if v.search(term, field):
                return v
        raise KeyError((term, field))

class DemutatingDict(DeepsearchingDict[S, T]):
    def __getitem__(self, key):
        try:
            return self.data.__getitem__(key)
        except KeyError as e:
            original_error = e

        demutated_key = Opers.Demutate(key)

        try:
            return super().__getitem__(demutated_key)
        except KeyError:
            # Raise the original exception
            raise original_error

# https://github.com/python/typing/issues/60
if TYPE_CHECKING:
    DictType = DeepsearchingDict[str, T]
else:
    DictType = DeepsearchingDict

class DatabaseDictionary:

    def __init__(self, dict_cls: Optional[Callable[..., DictType]] = None):
        if not dict_cls:
            dict_cls = DeepsearchingDict
        self.dict_cls = dict_cls
        self.noun: DictType[Noun] = dict_cls()
        self.adjective: DictType[Adjective] = dict_cls()
        self.preposition: DictType[Preposition] = dict_cls()
        self.nounPhrase: DictType[NP] = dict_cls()
        self.verb: DictType[Verb] = dict_cls()
        self.dicts = list(ENTITY_TYPE_MAP.keys())

    def search(self, entity, key):
        return self[entity].search(key, None)

    def __getitem__(self, key):
        # Allow retrieval by more standardized UPOS system
        if key in UPOS_TYPE_MAP:
            key = UPOS_TYPE_MAP[key]

        if key not in self.dicts:
            raise KeyError()

        return self.__dict__[key]

    def keys(self) -> Iterable[str]:
        return self.dicts

    def values(self) -> Iterable[dict[str, EntityType]]:
        return (
            self.__dict__[k]
            for k in
            self.dicts
        )

    def items(self) -> Iterable[tuple[str, dict[str, EntityType]]]:
        return (
            (k, self.__dict__[k])
            for k in
            self.dicts
        )

class ExtendedDatabaseDictionary(DatabaseDictionary):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.verbalNoun: DictType[Verb] = self.dict_cls()
        self.verbalAdjective: DictType[Verb] = self.dict_cls()
        self.dicts += ['verbalNoun', 'verbalAdjective']

class Database:
    def __init__(self, data_location: str, demutate: bool = False):
        self.data_location: str = data_location
        self.demutate = demutate
        self._dictionary: Optional[DatabaseDictionary] = None

    def load(self, database_dictionary_cls: Type[DatabaseDictionary]=DatabaseDictionary) -> None:
        entities: dict[str, type[EntityType]] = ENTITY_TYPE_MAP
        dict_cls = DemutatingDict if self.demutate else None
        self._dictionary = database_dictionary_cls(dict_cls=dict_cls)
        for folder, entity_type in entities.items():
            root = os.path.join(self.data_location, folder)
            for fn in os.listdir(root):
                word: EntityType = entities[folder].create_from_xml(os.path.join(root, fn))
                self._dictionary[folder][word.getLemma().lower()] = word
        supplementary_dir = os.path.join(os.path.dirname(__file__), 'supplementary')
        for fl in os.listdir(supplementary_dir):
            xml = ET.parse(os.path.join(supplementary_dir, fl))
            root_node = xml.getroot()
            word = self._dictionary[root_node.tag][root_node.get('default')]
            base_xml = word.printXml()
            base_xml_root = base_xml.getroot()
            base_xml_root.extend(root_node.iterchildren())
            self._dictionary[root_node.tag][root_node.get('default')] = entities[root_node.tag].create_from_xml(base_xml)

    @property
    def dictionary(self) -> DatabaseDictionary:
        if not self._dictionary:
            raise RuntimeError('Load dictionary first')
        return self._dictionary

    def __iter__(self) -> Generator[tuple[str, str, EntityType], None, None]:
        for pos, dct in self.dictionary.items():
            for lemma, word in dct.items():
                yield pos, lemma, word

    def __getitem__(self, pair: tuple[str, str]) -> EntityType:
        return self.dictionary[pair[0]][pair[1]]

    def __len__(self) -> int:
        return sum(map(len, self.dictionary.values()))

class SmartDatabase(Database):
    dictionary: ExtendedDatabaseDictionary

    def load(self):
        super().load(database_dictionary_cls=ExtendedDatabaseDictionary)
        self.dictionary.verbalNoun = {
            v.verbalNoun[0].value: v for v in self.dictionary.verb.values()
            if v.verbalNoun
        }

    def __getitem__(self, pair: tuple[str, str]) -> EntityType:
        try:
            return super().__getitem__(pair)
        except KeyError:
            pass

        # Perhaps we have a verbal noun/adjective
        entity = UPOS_TYPE_MAP.get(pair[0], pair[0])
        if not entity:
            raise KeyError(pair)

        if entity == 'noun':
            try:
                return self.dictionary.verbalNoun[pair[1]]
            except KeyError:
                pass
        raise KeyError(pair)

if __name__ == "__main__":
    database = Database(sys.argv[1])
    database.load()
    print(len(database), 'entries')
