from lxml import etree as ET
from gramadan.adjective import Adjective

# With thanks to BuNaMo
# https://github.com/michmech/BuNaMo
ADJECTIVES_XML = {
    'beo': """
        <adjective default="beo" declension="3" disambig="">
          <sgNom default="beo" />
          <sgGenMasc default="beo" />
          <sgGenFem default="beo" />
          <plNom default="beo" />
          <graded default="beo" />
        </adjective>
    """
}

def adjectives_xml():
    adjectives = {
        lemma: ET.ElementTree(ET.fromstring(xml))
        for lemma, xml in
        ADJECTIVES_XML.items()
    }

    return {
        lemma: Adjective.create_from_xml(xml)
        for lemma, xml in
        adjectives.items()
        if isinstance(xml, ET._ElementTree)
    }

ADJECTIVES_FROM_XML = adjectives_xml()
