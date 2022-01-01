import os
import pytest

RELATIVE_PATH_TO_PYTHON = os.path.join(
    os.path.dirname(__file__),
    os.pardir,
    os.pardir,
    'output'
)

RELATIVE_PATH_TO_CSHARP = os.path.join(
    os.path.dirname(__file__),
    os.pardir,
    os.pardir,
    os.pardir,
    'Tester'
)

TEST_FILES = (
    'test-poss-np.txt',
    'test-prep-poss-np.txt',
    'consistency-vns.txt',
    'consistency-report-adjectives.txt',
    'consistency-report-nouns.txt',
    'consistency-report-similarity.txt',
    'ainmfhocail-bearnaí.txt',
    'aidiachtaí-bearnaí.txt',
    'briathra-infinideacha-bearnaí.txt',
    'briathra-finideacha-bearnaí.txt',
    'OUTPUT.txt'
)

def unescape(text):
    text = text.replace("&amp;", "&");
    text = text.replace("&quot;", "\'");
    text = text.replace("&apos;", "'");
    text = text.replace("&lt;", "<");
    text = text.replace("&gt;", ">");
    return text

def test_compare_neid():
    csharp = os.listdir(os.path.join(RELATIVE_PATH_TO_CSHARP, os.pardir, 'NeidOutput'))
    for extra in ('!gram.css', '!gram.xsl'):
        try:
            csharp.remove(extra)
        except:
            pass
    python = os.listdir(os.path.join(RELATIVE_PATH_TO_PYTHON, 'NeidOutputPy'))

    assert csharp == python

    # XML has formatting differences that would
    # require more adjustment to align (or comparing objects,
    # which moves away from non-language-specific checking)
    for xml in csharp:
        with open(os.path.join(RELATIVE_PATH_TO_CSHARP, os.pardir, 'NeidOutput', xml), 'r', encoding='utf-8-sig') as f, \
                open(os.path.join(RELATIVE_PATH_TO_PYTHON, 'NeidOutputPy', xml), 'r') as g:
            left = f.readlines()
            right = g.readlines()
            pairs = zip(left, right)
            for n, (left, right) in enumerate(pairs):
                assert left.strip() == unescape(right.strip()), (xml, n)

@pytest.mark.parametrize('testfile', TEST_FILES)
def test_compare_output(testfile):
    with open(os.path.join(RELATIVE_PATH_TO_PYTHON, testfile), 'r') as f, \
            open(os.path.join(RELATIVE_PATH_TO_CSHARP, testfile), 'r') as g:
        left = f.readlines()
        right = g.readlines()
        pairs = zip(left, right)
        for n, (left, right) in enumerate(pairs):
            assert left.strip() == unescape(right.strip()), ('test-poss-np', n)

if __name__ == "__main__":
    test_compare_neid()
