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

**99.62%** (12329/12376). No `None` returns — all lemmas produce a form.

Errors by declension:

| Declension | Errors | Notes |
|---|---|---|
| 1st | 0 | |
| 2nd | 0 | |
| 3rd | 1 | `cion` (ambiguous) |
| 4th | 0 | |
| 5th | 46 | Irregular pluralised genitives (`abhainn→abhann`, `máthair→máthar`, etc.) |

## Verbs

### Conjugation class guessing

**100%** (3359/3359) using lemma-only heuristic.

### Paradigm generation

**99.97%** form-level (187240/187293). **99.52%** perfect verbs (3331/3347).

53 mismatches fall into three categories:

#### BuNaMo errors (9 mismatches, 2 verbs)

Cross-checked against [teanglann.ie](https://www.teanglann.ie/en/gram/) and
[An Caighdeán Oifigiúil §5.2.1(b)](https://irishlanguage.ie/an-caighdean-oifigiuil-chapter-5-the-verb/):

| Lemma | Errors | Reason |
|---|---|---|
| `sleacht` | 8 | BuNaMo uses `sleachttá` (double t after `cht`), but the Caighdeán rule §5.2.1(b) says `-t(h)t-` → `-t-` for roots ending in `-t`. BuNaMo's own `nocht`/`tacht` (same `-cht` ending) merge correctly. |
| `graf` | 1 | BuNaMo imper.auto = `grafatar` (extra `a`), but compound `liteagraf` uses standard `liteagraftar`. Only the imperative autonomous slot is affected. |

#### Caighdeán-over-BuNaMo: deighil future/conditional (18 mismatches)

Following [gnag](https://nualeargais.ie/gnag/typ1.htm) and Wiktionary:
`deighil` uses the **full lemma** (not syncopated stem) for future and
conditional forms: `deighilfidh` not `*deighlfidh`. Lars Bräsicke consistently
uses this form; Wiktionary agrees. All other deep syncope verbs follow BuNaMo.

#### Caighdeán-over-BuNaMo: -éigh cond.pl1 (28 mismatches, 14 verb entries)

BuNaMo contracts é-root cond.pl1 to `-fimis` (e.g. `léfimis`).
gnag [§typ1igh](https://nualeargais.ie/gnag/typ1igh.htm) gives `-ifimis`
(e.g. `léifimis`), consistent with the standard paradigm pattern.
Zero attestation for BuNaMo's contracted forms.

### Verbal adjective generation

**100%** (3346/3346). All h-prothesis loanword forms and `comhair`/`tamhain`
edge cases handled via exact overrides.

### Verbal noun generation

**100%** (3346/3346). VN is far more irregular than VA (~140 distinct
formation patterns). Achieved via:

- Rule-based generation for regular 1st and 2nd conjugation verbs
- Suffix-matched overrides (longest match) for compound verb families
  (e.g., all `-scríobh` compounds → VN=lemma, all `-loisc` → `-loscadh`)
- Exact-match overrides for ~130 standalone irregular verbs

Key linguistic patterns:

| Pattern | Example | Count |
|---|---|---|
| VN=lemma | `scríobh`, `snámh`, `rith` | ~70 |
| -igh strip (long vowel root) | `dóigh→dó`, `spréigh→spré` | ~25 |
| -aigh→-ú (2nd conj regular) | `achtaigh→achtú` | ~1200 |
| -igh→-iú (2nd conj regular) | `Laidinigh→Laidiniú` | ~400 |
| -ir/-il→broaden+adh | `codail→codladh`, `bodhair→bodhradh` | ~50 |
| -isc→broaden+scadh | `loisc→loscadh`, `fáisc→fáscadh` | ~15 |
| -scaoil→-scaoileadh | `scaoil→scaoileadh` | ~9 |
| h-prothesis (BuNaMo convention) | `aicleáil→haicleáil` | 4+5 |
