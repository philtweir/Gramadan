import sys
import re
import logging
import os

from gramadan.noun import Noun
from gramadan.features import Gender, FormSg
from gramadan.opers import Opers
from gramadan.singular_info import SingularInfoA, SingularInfoE, SingularInfoC, SingularInfoL, SingularInfoAX, SingularInfoEAX, SingularInfoD, SingularInfoN
from gramadan.v2.database import Database
from gramadan.v2.semantic_groups import FAMILY
from gramadan.v2.other_groups import POSSIBLE_LOANWORDS_GENITIVELESS, BUNAMO_ONLY_GENITIVELESS

class DeclensionInconsistentError(Exception):
    def __init__(self, msg, dec, dec_exp):
        super().__init__(msg)
        self.dec = dec
        self.dec_exp = dec_exp

class FormsMissingException(Exception):
    pass

class FormsAmbiguousException(Exception):
    pass

class NounDeclensionGuesser:
    MULTIPLE_WORDS = {
        'sail': ((5, 'saileach'), (2, 'saile')),
        'cian': ((1, 'cian'), (2, 'céine'))
    }

    # These are not only unexpectedly included
    # into a declension, but would match a different
    # declension if they were not noted.
    FULLY_IRREGULAR = {
            'laghad': (1, 'laghad'),
            # BuNaMo says 1 and AFB says 4 - AFB may seem
            # more consistent, but we are treating BuNoMo
            # as source of truth for this guesser.

            'anachain': (3, 'anachaine'),
            # BuNaMo says 3 and AFB says 2
            'leann': (1, 'leanna'),
            # BuNaMo says 1 and AFB says 3
            'bunáite': (2, 'bunáite'),
            # BuNaMo says 2 and AFB says 4
            'troitheán': (4, 'troitheáin'),
            # BuNaMo says 4 and AFB says 2
            'onnmhaireoir': (4, 'onnmhaireora'),
            # BuNaMo says 4 and AFB says 3

            'gínéiceolaíocht': (3, 'gínéiceolaíocht'),
            # BuNaMo says 3 but Tearma says 3 w
            # gínéiceolaíochta as gen.

            'réamhghlacan': (2, 'réamhghlacana'),
            # Otherwise gets seen as 3rd. Should glacan move
            # too?

            'ardaicme': (2, 'ardaicme'),
            # Otherwise gets seen as 4th. Odd as aicme 4th.

            'fobhóthar': (1, 'fobhóthar'),
            # Otherwise gets seen as 4th.
            # Odd as gen. bóthar (dec1) is bóthair.

            'righneáil': (2, 'righneála'),
            # BuNaMo says 2 and AFB says 3

            'catacóm': (1, 'catacóma'),

            'spásrás': (1, 'spásrása'),
            # Odd as rás 3rd

            'cúblálaí': (3, 'cúblálaí'),
            # BuNaMo says 3 and AFB says 4

            'slogadh': (1, 'slogtha'),

            'cionroinnt': (2, 'cionranna'),

            # (as well as certain of the above)
            # The genitives of the below do not
            # follow any of the singular-genitive-making
            # rules for that declension.
            # (Would be keen to understand what includes
            #  them in the respectives declensions)

            # First
            'fuineadh': (1, 'fuinte'),
            'cian': (1, 'cian'),
            'muineál': (1, 'muiníl'),
            'ainchleachtadh': (1, 'ainchleachta'),
            'bonnbhualadh': (1, 'bonnbhuailte'), # presumably also buailte
            'díraonadh': (1, 'díraonta'),
            'leabharchoimeád': (1, 'leabharchoimeádta'),
            'domhainfhriochtán': (1, 'domhanfhriochtáin'),
            # This really seems like a typo, given that domhainfhriochtóir
            # and domhainmhachnamh form a regular genitive without domhain->domhan
            'Malaech': (1, 'Malaeich'),

            # It's actually 4 going to aoich, and 4 going
            # to aoigh (cianghlaioch, glaoch, fraoch, naoch)
            'laoch': (1, 'laoich'),
            'caoch': (1, 'caoich'),
            'éicealaoch': (1, 'éicealaoich'),
            'crannlaoch': (1, 'crannlaoich'),

            'dobhareach': (1, 'dobhareich'),
            'each': (1, 'eich'),

            'stóch': (1, 'stóich'),
            'cóch': (1, 'cóich'),
            'fíoch': (1, 'fích'),


            'ochtach': (1, 'ochtaí'),
            'bearach': (1, 'bearaí'),

            'Gael': (1, 'Gaeil'),

            'taghd': (1, 'taghaid'), # FGB disagrees, says gen is taighd (reg)

            # Second
            'fidil': (2, 'fidle'),
            'scian': (2, 'scine'),

            'sliabh': (2, 'sléibhe'),
            'bansliabh': (2, 'bansléibhe'),
            'droimshliabh': (2, 'droimshléibhe'),
            'blocshliabh': (2, 'blocsléibhe'),

            'loilíoch': (2, 'loilí'),

            # Third
            'prios': (3, 'priosa'),
            'goid': (3, 'gada'),
            'conradh': (3, 'conartha'),
            'cumhachtroinnt': (3, 'cumhachtroinnte'),
            'bunmhúinteoir': (3, 'bunmhuinteora'), # _Surely_ this is a BuNaMo typo
            'cion': (3, 'ciona'),
            'cosaint': (3, 'cosanta'),
            'dioc': (3, 'dioca'),
            'siorc': (3, 'siorca'),
            'giolc': (3, 'giolca'),
            'triuch': (3, 'treacha'),
            'miocht': (3, 'miochta'),
            'toirbhirt': (3, 'toirbhearta'),
            'iarmhairt': (3, 'iarmharta'),
            'mionn': (3, 'mionna'),
            'méadail': (3, 'méadla'), # this could be in syncopation, but dl->ll (should it then for SIA?)
            'iarraidh': (3, 'iarrata'),
            'muir': (3, 'mara'),
            'mallmhuir': (3, 'mallmhara'),
            'seachaint': (3, 'seachanta'),

            # Fourth
            'dháréag': (4, 'dáréag'),
            'ionsaí': (4, 'ionsaithe'),
            'spré': (4, 'spréite'),
            'dreo': (4, 'dreoite'),
            'araí': (4, 'araíon'),
            'coinnealbhá': (4, 'coinnealbháite'), # strange as other 4Fem compounds with bhá are reg.

            # Fifth
            'fearchú': (5, 'fearchon'),
            'árchú': (5, 'árchon'),
            'onchú': (5, 'onchon'),
            'sail': (5, 'saileach'), # cf. sail - also fem, dec.2
            'caora': (5, 'caorach'), # irreg. because not -n/l/r
            'cathaoir': (5, 'cathaoireach'),
            'fóir': (5, 'fóireach'),
            'cara': (5, 'carad'),
            'namhaid': (5, 'namhad'),
            'bráid': (5, 'brád'),
            'Nollaig': (5, 'Nollag'),
    }

    IRREGULAR_INCLUSION = {
            'straidhn': 2,
            'tiúin': 2,
            'spairn': 2,
            'diathair': 2,
            'deoir': 2,
            'éitir': 2,
            'cosmhuintir': 2,
            'muintir': 2,
            'aibítir': 2,
            'vacsaín': 2,
            'dín': 2,

            'mórphearsa': 4,
            'gabhair': 4,
            'feac': 4,
            'cadhc': 4,
            'faic': 4,
            'feic': 4,
            'sonc': 4,
            'bunc': 4,
            'hurlamaboc': 4,
            'trioc': 4,
            'stad': 4,
            'leathstad': 4,
            'grianstad': 4,
            'lánstad': 4,
            'idirstad': 4,
            'leagáid': 4,
            'iarmhéid': 4,
            'uasmhéid': 4,
            'bréid': 4,
            'triuf': 4,
            'dallamullóg': 4,
            'tiubh': 4,
            'neach': 4,
            'trach': 4,
            'bláthchuach': 4,
            'imchuach': 4,
            'cách': 4,
            'fáidh': 4,
            'príomháidh': 4,
            'aodh': 4,
            'anraith': 4,
            'líonrith': 4,
            'clíth': 4,
            'ceal': 4,
            'meal': 4,
            'Iúil': 4,
            'pléiseam': 4,
            'seachain': 4,
            'cliamhain': 4,
            'banchliamhain': 4,
            'dóthain': 4,
            'Céadaoin': 4,
            'Déardaoin': 4,
            'lachín': 4,
            'beainín': 4,
            'óinsín': 4,
            'bóín': 4,
            'lao': 4,
            'glao': 4,
            'trup': 4,
            'seáp': 4,
            'lear': 4,
            'sciar': 4,
            'riar': 4,
            'paor': 4,
            'gearr': 4,
            'seacht': 4,
            'ocht': 4,
            'ceant': 4,

            'leaca': 5,
            'ionga': 5,
            'Lemma': 5,
            'céimseata': 5,
            'leite': 5,
            'cabhail': 5,
            'traein': 5,
            'cáin': 5,
            'mótarcháin': 5,
            'cráin': 5,
            'coróin': 5,
            'abhainn': 5,
            'siocair': 5,
            'carcair': 5,
            'dair': 5,
            'corcdhair': 5,
            'láthair': 5,
            'lasair': 5,
            'loinnir': 5,
            'láir': 5,
            'céir': 5,
            'fíor': 5,
            'deachú': 5,
            'cian': 5,
    }


    IRREGULAR_GROUPS = (
            lambda l, g: 5 if l in FAMILY else None, # Family
    )

    # This is not simply nouns that are in unexpected
    # declensions, but the nouns in the specific sixth
    # "Irregular Declension" of miscellaneous entries.
    IRREGULAR_DECLENSION = {
            'bean': 'mná',
            'deirfiúr': 'deirfear',
            'deoch': 'dí',
            'siúr': 'siúrach',
            'dia': 'dé',
            'lá': 'lae',
            'leaba': 'leapa',
            'mí': 'míosa',
            'olann': 'olla',
            'talamh': 'talún', # Also talaimh according to An Caighdeán
            'ó': 'uí', # thanks to Nualeargais
            # teach according to An Caighdeán, see below.
    }

    # Teach family is a funny one - according to Nualeargais,
    # teach is in the 2nd Declension. BuNaMo doesn't say (which
    # can mean irregular declension or simply not assigned), and
    # An Caighdeán says Irregular.
    #
    # However, BuNaMo puts two teach derivatives into 1st Decl and
    # omits info about any of the rest (including teach itself).
    #      craobhteach and díonteach
    # Note that this is not simply masculine nouns ending in teach,
    # these all have genitives formed with -t(h)í, and seem to somehow
    # relate to houses/buildings.
    TEACH_FAMILY = (
        'cloigtheach',
        'clubtheach',
        'cléirtheach',
        'coirmtheach',
        'cruinnteach',
        'cúlteach',
        'dairtheach',
        'fiailteach',
        'fleiteach',
        'fortheach',
        'longtheach',
        'mainteach',
        'máthairtheach',
        'plódteach',
        'proinnteach',
        'rítheach',
        'teach',
        'túrtheach',
        'urtheach',
        'íolteach',
    )
    TEACH_FAMILY_1ST = (
        'craobhtheach',
        'díonteach',
    )

    def guess(self, focal: Noun):
        if len(focal.sgNom) != 1 or len(focal.sgGen) != 1:
            raise FormsMissingException(
                f'{focal.getLemma()} has {len(focal.sgNom)} nom and {len(focal.sgGen)} gen'
            )

        lemma = focal.getLemma()
        gender = focal.getGender()

        if lemma in self.IRREGULAR_DECLENSION:
            if focal.declension and focal.declension in (1, 2, 3, 4, 5):
                if focal.declension == 5:
                    logging.warning(f'{lemma}: BuNaMo puts some irregular nouns as Dec.5 (e.g. deirfúir)')
                    return 5
                raise DeclensionInconsistentError(
                    f'Declension not determined as expected for {lemma}! Expected {focal.declension}',
                    -1, focal.declension
                )
            return -1

        if lemma in self.MULTIPLE_WORDS:
            match = [
                dec for dec, gen in self.MULTIPLE_WORDS[lemma]
                if gen == focal.sgGen[0].value
            ]
            if len(match) == 1:
                return match[0]
            elif not match:
                raise DeclensionInconsistentError(
                    f'Could not match any for multi-declension {lemma}!',
                    self.MULTIPLE_WORDS[lemma], focal.declension
                )
            else:
                raise FormsAmbiguousException(
                    f'Matched {len(match)} allowed declensions for {lemma}'
                )

        if lemma in self.FULLY_IRREGULAR:
            dec, gen = self.FULLY_IRREGULAR[lemma]
            if focal.sgGen[0].value != gen:
                raise DeclensionInconsistentError(
                    f'Hard-coded genitive for dec{dec} not expected for {lemma}!',
                    dec, focal.declension
                )
            if focal.declension and focal.declension != dec:
                raise DeclensionInconsistentError(
                    f'Hard-coded declension {dec} not expected for {lemma}! Expected {focal.declension}',
                    dec, focal.declension
                )
            return dec

        for matcher in self.IRREGULAR_GROUPS:
            if (mdec := matcher(lemma, gender)) is not None:
                return mdec

        # We treat the teach family separately.
        if lemma in self.TEACH_FAMILY:
            return -1
        elif lemma in self.TEACH_FAMILY_1ST:
            # This makes very little sense, when An Caighdeán
            # Oifigiúil puts teach in the Irregulars, but we
            # follow BuNaMo, and these words are only derivatives
            # of teach.
            return 1

        checks = [lambda _: _,
            self._is_first,
            self._is_second,
            self._is_third,
            self._is_fourth,
            self._is_fifth,
        ]
        for dec in range(5, 0, -1):
            irregular_inclusion = self.IRREGULAR_INCLUSION.get(lemma, None)
            if irregular_inclusion == dec or (not irregular_inclusion and checks[dec](focal)):
                # if irregular_inclusion == dec:
                #     checks[dec](focal) # So we can get genitive
                return dec

        return None

    # The below are private as to save
    # duplication, they assume they have been run
    # in the order above.

    def _is_fourth(self, focal: Noun):
        lemma = focal.getLemma()
        gender = focal.getGender()

        irregularly_palatalized: dict[str, str] = {}

        if lemma in POSSIBLE_LOANWORDS_GENITIVELESS + BUNAMO_ONLY_GENITIVELESS:
            return True

        # Should we even be here?
        if (
                lemma.endswith('ín') and gender is Gender.Masc

                or

                gender is Gender.Masc and
                (
                    (
                        {lemma[-i:] for i in range(1, 4)} & {'ín', 'aí', 'ú', 'nm', 'iam', 'cs', 'ts', 'ns', 'eo'}
                        or re.search('[^óoé]ir$', lemma)
                    )
                    and not re.search('[eú]ir', lemma)
                    and lemma not in FAMILY # Masc family are usu. 5th
                )

                or

                {lemma[-i:] for i in range(1, 4)} & {'a', 'e', 'í', 'le', 'ne', 'é', 'aoi', 'ó', 'á'}
            ):
            return True

        # This doesn't seem to be clearly documented in the
        # usual places, but 49 nouns ending in ú in BuNaMo
        # are marked as 4th declension, but the genitive is
        # different from nominative. This compares to 105 that
        # end in ú, are marked 4th and gen=nom. All of both sets
        # are masculine, in contrast to 2nd decl.
        #if gender is Gender.Masc:
        #    palatalized = SingularInfoE(
        #        lemma,
        #        focal.getGender(),
        #        syncope=True,
        #        doubleDative=False,
        #        slenderizationTarget=(
        #            irregularly_palatalized.get(lemma, "")
        #        ),
        #        v2=True
        #    )

        #    if palatalized.genitive[0].value == focal.sgGen[0].value:
        #        return True

        if focal.declension and focal.declension == 4:
            raise DeclensionInconsistentError(
                f'Declension not determined as expected for {lemma}! Expected {focal.declension}',
                4, focal.declension
            )

        return False

    def _is_second(self, focal: Noun):
        lemma = focal.getLemma()
        gender = focal.getGender()

        monosyllabic_i = {
            'cearc',
            'beanne',
            'beann',
        }
        polysyllabic_ei = {
            'meadar',
            'maoildearg',
            'taibhdhearc',
            'seamair',
            'díthreabh',
            'taibhdhearc',
            'aershreabh',
            'bláthfhleasc',
            'bonnleac',
            'cráinbheach',
            'díthreabh',
        }
        palatalized = SingularInfoE(
            lemma,
            focal.getGender(),
            syncope=False,
            doubleDative=False,
            slenderizationTarget=(
                "ei" if lemma in polysyllabic_ei else ""
            ),
            v2=True,
            with_monosyllabic_ei_v2=lemma not in monosyllabic_i
        )

        if palatalized.genitive[0].value == focal.sgGen[0].value:
            return True

        palatalized_sync = SingularInfoE(
            focal.sgNom[0].value,
            focal.getGender(),
            syncope=True,
            doubleDative=False,
            slenderizationTarget=(
                "ei" if lemma in polysyllabic_ei else ""
            ),
            v2=True,
            with_monosyllabic_ei_v2=lemma not in monosyllabic_i
        )

        if palatalized_sync.genitive[0].value == focal.sgGen[0].value:
            return True

        # This could be at the top, but the farther we can go
        # without gender, the better our guessing without it.
        if gender == Gender.Fem:
            palatalized_c = SingularInfoC(
                focal.sgNom[0].value,
                focal.getGender(),
                slenderizationTarget=(
                    "ei" if lemma in polysyllabic_ei else ""
                ),
                v2=True
            )

            if palatalized_c.genitive[0].value == focal.sgGen[0].value:
                return True

        if focal.declension and focal.declension == 2:
            raise DeclensionInconsistentError(
                f'Declension not determined as expected for {lemma}! Expected {focal.declension}',
                2, focal.declension
            )
        return False

    def _is_third(self, focal: Noun):
        lemma = focal.getLemma()
        gender = focal.getGender()

        unsyncopated_exceptions = (
            'athbhliain',
            'bliain',
            'bonnbhliain',
            'idirbhliain',
            'scoilbhliain',
            'solasbhliain',

            'argain',
            'cluain',
            'dúnorgain',
            'foluain',
            'fothain',

            'féachaint',
            'súilfhéachaint',
            'tuargaint',

            'marthain',

            'cantain',
            'cianrochtain',
            'comhriachtain',
            'dámhachtain',
            'fionnachtain',
            'mainneachtain',
            'rochtain',
        )

        depalatalized = SingularInfoA(
            lemma,
            gender,
            syncope=False,
            v2=True,
            with_syncopated_ai_v2=lemma not in unsyncopated_exceptions,
        )

        if depalatalized.genitive[0].value == focal.sgGen[0].value:
            return True

        if focal.declension and focal.declension == 3:
            raise DeclensionInconsistentError(
                f'Declension not determined as expected for {lemma}! Expected {focal.declension}',
                3, focal.declension
            )
        return False

    def _is_first(self, focal: Noun):
        lemma = focal.getLemma()

        irregularly_palatalized = {
            'mac': 'i',
            'garmhac': 'i',

            'bligeard': 'eai',
            'seac': 'eai',

            'earc': 'ei',
            'gaistreintríteas': 'ei',
            'dearg': 'ei',
            'meadhg': 'ei',
            'corcairdhearg': 'ei',
            'infridhearg': 'ei',

            'Eiritréach': 'éai',
        }

        without_iai = {
            'cliabh',
            'fiach',
            'fial',
            'giall',
        }
        palatalized = SingularInfoC(
            lemma,
            focal.getGender(),
            slenderizationTarget=(
                irregularly_palatalized.get(lemma, "")
            ),
            v2=True,
            with_iai_v2=lemma not in without_iai
            # nearly all Dec.1 nouns with ia as final
            # vowel cluster become iai, except for
            # four in BuNaMo (and any ending iasc, which
            # are handled in SingularInfoC).
        )

        if palatalized.genitive[0].value == focal.sgGen[0].value:
            return True

        if focal.declension and focal.declension == 1:
            raise DeclensionInconsistentError(
                f'Declension not determined as expected for {lemma}! Expected {focal.declension}',
                1, focal.declension
            )
        return False

    def _is_fifth(self, focal: Noun):
        lemma = focal.getLemma()
        gender = focal.getGender()
        depalatalized = SingularInfoL(lemma, gender, v2=True)

        if gender == Gender.Masc:
            return False

        if lemma.endswith('ceathrú') or lemma.endswith('cheathrú'):
            return True

        if depalatalized.genitive[0].value == lemma and not re.search('a(rs|ch|rch|nm)a$', lemma):
            return False

        if gender == Gender.Fem:
            if focal.sgNom[0].value[-1] in ('r', 'l', 'n'):
                syncopated_ax = SingularInfoAX(lemma, gender, syncope=False, v2=True)
                if (
                    {lemma[-i:-1] for i in range(2, 6)} & {'riai', 'tiúi'} or
                    {lemma[-i:] for i in range(1, 10)} & {'thir', 'tir', 'mhir', 'eoir', 'athair', 'ochair', 'bhair', 'eorainn'}
                ):
                    return True

                if (
                    {lemma[-i:-1] for i in range(2, 4)} & {'í', 'úi', 'ói', 'éi', 'in', 'ái'} or
                    {lemma[-i:] for i in range(1, 7)} & {'eoil', 'coil', 'tir', 'bair', 'ain', 'ir', 'ill', 'il', 'in'}
                ):
                    return False

                return True
                if syncopated_ax.genitive[0].value == focal.sgGen[0].value:
                    return True

                palatalized_ax = SingularInfoAX(lemma, gender, syncope=True, v2=True)

                if palatalized_ax.genitive[0].value == focal.sgGen[0].value:
                    return True

                syncopated = SingularInfoEAX(lemma, gender, syncope=True, v2=True)

                if syncopated.genitive[0].value == focal.sgGen[0].value:
                    return True
            elif focal.sgNom[0].value[-1] in Opers.Vowels:
                modified_n = SingularInfoN(lemma, gender)

                #if modified_n.genitive[0].value == focal.sgGen[0].value:
                return True
        # Doesn't seem necessary...
        #elif gender == Gender.Masc and focal.sgNom[0].value[-1] in Opers.Vowels:
        #    modified_d = SingularInfoD(lemma, gender)

        #    if modified_d.genitive[0].value == focal.sgGen[0].value:
        #        return True

        if focal.declension and focal.declension == 5:
            raise DeclensionInconsistentError(
                f'Declension not determined as expected for {lemma}! Expected {focal.declension}',
                5, focal.declension
            )
        return False

# Thanks to http://www.nualeargais.ie/foghlaim/nouns.php for the rules
def _find_f4_nouns(database):
    for noun in database.dictionary['noun'].values():
        lemma = noun.getLemma()
        if lemma[-1] in ('e', 'í') and noun.getGender() == Gender.Fem:
            yield lemma

if __name__ == "__main__":
    guesser = NounDeclensionGuesser()
    if len(sys.argv) > 1:
        path = sys.argv[1]
    else:
        path = os.path.join(os.path.dirname(__file__), os.pardir, os.pardir, 'output', 'data')

    dictionary = None
    print_result = False
    for n, arg in enumerate(sys.argv):
        if arg == '--word':
            lemma, gender, gen = sys.argv[n+1].split(',')
            noun = Noun(
                [FormSg(lemma, Gender(gender))],
                [FormSg(gen, Gender(gender))],
            )
            dictionary = {
                lemma: noun
            }
            print_result = True

    if not dictionary:
        database = Database(path)
        database.load()
        dictionary = database.dictionary.noun

    errors = []
    known = 0
    unknown = []
    for n, (lemma, noun) in enumerate(dictionary.items()):
        try:
            if noun.declension:
                known += 1
            dec = guesser.guess(noun)
            if noun.declension:
                if dec != noun.declension:
                    raise DeclensionInconsistentError(
                        f'Declension for {lemma}, expected {noun.declension}, got {dec}',
                        dec,
                        noun.declension
                    )
            else:
                noun.declension = dec
                if print_result and not dec:
                    unknown.append(noun)
        except FormsMissingException as e:
            if '--debug' in sys.argv:
                print('[', e, ']')
        except DeclensionInconsistentError as e:
            errors.append((noun, e))
            if '--debug' in sys.argv:
                print(e)

    if len(errors) < 20 or '--all' in sys.argv:
        import tabulate # type: ignore
        table = [
            {
                'Lemma': noun.getLemma(),
                'Gender': noun.getGender().value,
                'Genitive': noun.sgGen[0].value if len(noun.sgGen) else '',
                'Error': str(e)
            }
            for noun, e in errors
        ]
        print(tabulate.tabulate(table, headers='keys'))

    if print_result:
        import tabulate # type: ignore
        table = [
            {
                'Lemma': lemma,
                'Gender': noun.getGender().value,
                'Genitive': noun.sgGen[0].value if len(noun.sgGen) else '',
                'Declension': 'IRR.' if noun.declension == -1 else noun.declension
            }
            for lemma, noun in dictionary.items()
        ]
        print(tabulate.tabulate(table, headers='keys'))
        if unknown:
            print(f"{len(unknown)} could not be guessed: {', '.join([n.getLemma() for n in unknown])}")
    if known:
        print(f"Errors {len(errors)} in {known} known, {100 * (1 - len(errors) / known):.2f}% success")
