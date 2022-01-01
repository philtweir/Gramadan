import pytest
from lxml import etree as ET

from gramadan.noun import Noun
from gramadan.features import Gender

# With thanks to BuNaMo
# https://github.com/michmech/BuNaMo
NOUNS_XML = {
        'clárú': """
            <noun default="clárú" declension="0" disambig="" isProper="0" isDefinite="0" allowArticledGenitive="0">
              <sgNom default="clárú" gender="masc" />
              <sgGen default="cláraithe" gender="masc" />
            </noun>
        """,
        'cléireach-chlóscríobhaí': """
            <noun default="cléireach-chlóscríobhaí" declension="0" disambig="" isProper="0" isImmutable="0" isDefinite="0" allowArticledGenitive="0">
              <sgNom default="cléireach-chlóscríobhaí" gender="masc" />
              <sgGen default="cléireach-chlóscríobhaí" gender="masc" />
              <plNom default="cléireach-chlóscríobhaithe" />
              <plGen default="cléireach-chlóscríobhaithe" strength="strong" />
            </noun>
        """,
        'Djibouti': """
            <noun default="Djibouti" declension="0" disambig="" isProper="1" isDefinite="1" allowArticledGenitive="0">
              <sgNom default="Djibouti" gender="fem" />
              <sgGen default="Djibouti" gender="fem" />
            </noun>
        """,
        'uair': """
            <noun default="uair" declension="2" disambig="" isProper="0" isDefinite="0" allowArticledGenitive="0">
              <sgNom default="uair" gender="fem" />
              <sgGen default="uaire" gender="fem" />
              <plNom default="uaireanta" />
              <plNom default="uaire" />
              <plGen default="uaireanta" strength="strong" />
              <plGen default="uaire" strength="strong" />
              <count default="uaire" />
            </noun>
        """
}

@pytest.fixture
def nouns():
    nouns = {
        lemma: ET.ElementTree(ET.fromstring(xml))
        for lemma, xml in
        NOUNS_XML.items()
    }

    return {
        lemma: Noun.create_from_xml(xml)
        for lemma, xml in
        nouns.items()
        if isinstance(xml, ET._ElementTree)
    }

def test_noun_clárú_behaves_as_expected(nouns):
    clárú = nouns['clárú']
    assert clárú.getLemma() == 'clárú'
    assert clárú.getGender() == Gender.Masc
    assert clárú.declension == 0
    assert len(clárú.sgNom) == 1
    assert clárú.sgNom[0].value == 'clárú'
    assert len(clárú.sgGen) == 1
    assert clárú.sgGen[0].value == 'cláraithe'
    assert len(clárú.sgVoc) == 0
    assert len(clárú.sgDat) == 1
    assert clárú.sgDat[0].value == 'clárú'
    assert len(clárú.plNom) == 0
    assert len(clárú.plGen) == 0
    assert len(clárú.plVoc) == 0
    assert len(clárú.count) == 0
    assert clárú.isProper is False
    assert clárú.isImmutable is False
    assert clárú.isDefinite is False
    assert clárú.allowArticledGenitive is False
    assert clárú.declension == 0
    assert clárú.disambig == ""
