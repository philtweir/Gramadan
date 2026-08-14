# gramadan-rs: Accuracy Caveats

Cross-validated against BuNaMo XML data (Gramadán project).

## Adjectives

### Declension guessing

**100%** (6634/6634) using 3 rules: `-úil` → 2nd, vowel ending → 3rd, else 1st.

### Form generation (sgGenMasc, sgGenFem, plNom, graded)

**100%** where answered (26514/26514). 3 lemmas return `None`:

| Lemma | Reason |
|---|---|
| `ceanntréan` | Compound éa→éai slenderization in sgGenMasc; BuNaMo expects `ceanntréain`, standard rule gives `ceanntréin` |
| `fíormhaith` | BuNaMo has sgGenFem = lemma (no change), but the 3 other `mhaith` compounds all use `mhaithe`; likely a BuNaMo inconsistency |
| `tréan` | Graded form has two accepted alternatives (`treise/tréine`); we produce `treise` |

## Nouns

### Declension guessing

**99.93%** where answered (12360/12369). 8 lemmas return `None` (genuinely ambiguous — two valid declensions depending on sense):

`beart`, `bran`, `cion`, `cruach`, `cuarc`, `lucht`, `léas`, `sail`

9 errors remain in the answered set (out of 12369).

### Genitive generation

**98.55%** (12196/12376). No `None` returns — all lemmas produce a form.

Errors by declension:

| Declension | Errors | Notes |
|---|---|---|
| 1st | 0 | |
| 2nd | 55 | Syncopation in compounds (`scuabobair`, `mótarfheithicil`, etc.) |
| 3rd | 2 | `cion` (ambiguous), `cuach` |
| 4th | 77 | Verbal noun genitives (`-ú` → `-aithe/-ithe`) not yet generated |
| 5th | 46 | Irregular pluralised genitives (`abhainn→abhann`, `máthair→máthar`, etc.) |

The 4th-declension errors are systematic: words ending in `-ú` (verbal nouns) whose genitive is the verbal adjective (`-aithe`/`-ithe`). These could be addressed with a suffix rule.

## Verbs

### Conjugation class guessing

**100%** (3359/3359) using lemma-only heuristic.

### Paradigm generation

**~100%** form-level (187284/187293). **99.94%** perfect verbs (3345/3347).

9 remaining mismatches are confirmed BuNaMo errors, cross-checked against
[teanglann.ie](https://www.teanglann.ie/en/gram/) and
[An Caighdeán Oifigiúil §5.2.1(b)](https://irishlanguage.ie/an-caighdean-oifigiuil-chapter-5-the-verb/):

| Lemma | Errors | Reason |
|---|---|---|
| `sleacht` | 8 | BuNaMo uses `sleachttá` (double t after `cht`), but the Caighdeán rule §5.2.1(b) says `-t(h)t-` → `-t-` for roots ending in `-t`. BuNaMo's own `nocht`/`tacht` (same `-cht` ending) merge correctly. |
| `graf` | 1 | BuNaMo imper.auto = `grafatar` (extra `a`), but compound `liteagraf` uses standard `liteagraftar`. Only the imperative autonomous slot is affected. |
