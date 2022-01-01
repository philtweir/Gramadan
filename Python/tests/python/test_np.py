from textwrap import dedent
from .test_noun import NOUNS_FROM_XML
from .test_adjective import ADJECTIVES_FROM_XML
from gramadan.np import NP
from gramadan.possessive import Possessive
from gramadan.features import Mutation

_NOUN_PHRASES = {
    'clárú beo': """
        sgNom: [clárú beo]
        sgGen: [cláraithe bheo]
        plNom:
        plGen:
        sgNomArt: [an clárú beo]
        sgGenArt: [an chláraithe bheo]
        plNomArt:
        plGenArt:

        sgDat: [clárú beo]
        sgDatArtN: [clárú bheo]
        sgDatArtS: [clárú beo]
        plDat:
        plDatArt:
    """,
    'mo clárú beo': """
        sgNom: [mo chlárú beo]
        sgGen: [mo chláraithe bheo]
        plNom:
        plGen:
        sgNomArt:
        sgGenArt:
        plNomArt:
        plGenArt:

        sgDat: [mo chlárú beo]
        sgDatArtN:
        sgDatArtS:
        plDat:
        plDatArt:
    """
}

NOUN_PHRASES = {
    lemma: '\n'.join([line + ' ' for line in table.split('\n')]) # this is just how the C# is formatted
    for lemma, table in _NOUN_PHRASES.items()
}

def test_clárú_beo():
    clárú = NOUNS_FROM_XML['clárú']
    beo = ADJECTIVES_FROM_XML['beo']

    np = NP.create_from_noun_adjective(clárú, beo)
    table = np.print()

    assert dedent(table).strip() == dedent(NOUN_PHRASES['clárú beo']).strip()

def test_mo_clárú_beo():
    mo = Possessive.create_from_string_and_mutation(
        'mo',
        Mutation.Len1
    )

    clárú = NOUNS_FROM_XML['clárú']
    beo = ADJECTIVES_FROM_XML['beo']

    np = NP.create_from_noun_adjective_possessive(clárú, beo, mo)
    table = np.print()

    assert dedent(table).strip() == dedent(NOUN_PHRASES['mo clárú beo']).strip()
