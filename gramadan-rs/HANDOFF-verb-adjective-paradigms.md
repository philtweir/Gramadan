# HANDOFF: verb + adjective paradigms — integration guide for Gréasán

**Status: COMPLETE.** `gramadan-rs` generates full paradigms for **verbs**
(conjugation + verbal noun + verbal adjective + shape rules) and
**adjectives** (declension + graded forms), matching what `noun.rs` does for
nouns. The WASM binding (`gramadan-wasm`) is wired and emits tagged
`FormItem[]` for all three POS types.

## What's built

| Component | Accuracy | Notes |
|---|---|---|
| Verb conjugation | 99.97% form-level (187240/187293) | 53 mismatches are BuNaMo errors or Caighdeán-over-BuNaMo choices (see CAVEATS.md) |
| Verbal adjective | 100% (3346/3346) | Two rules + 11 exact overrides |
| Verbal noun | 100% (3346/3346) | Three-tier dispatch: exact → longest-suffix → rule-based |
| Conjugation class guessing | 100% (3359/3359) | Lemma-only heuristic |
| Adjective declension | 100% (26514/26514) | 3 lemmas return `None` (genuine ambiguity) |
| Adjective class guessing | 100% (6634/6634) | 3 rules: `-úil`→2nd, vowel→3rd, else 1st |
| Shape rules (a/n) | Verified against py/v2 | Interrogative + negative for all tenses × persons |

All cross-validated against BuNaMo XML data.

## Rust API

### Verbs

```rust
use gramadan::verb::{Verb, VerbConjugationClass, guess_conjugation};

// From a known class:
let v = Verb::from_lemma("mol", VerbConjugationClass::First);

// Or guess the class from the lemma:
let class = guess_conjugation("ceannaigh");  // → Second
let v = Verb::from_lemma("ceannaigh", class);

// Principal parts:
let vn: &[Form] = &v.verbal_noun;      // e.g. "moladh"
let va: &[Form] = &v.verbal_adjective;  // e.g. "molta"

// Tense paradigms (each has .indep, .dep, .rel_indep → PersonForms):
let past_sg1 = &v.past.indep.sg1;   // Vec<Form>
let fut_auto = &v.fut.indep.auto;    // autonomous

// Shape rules (particle + mutated dependent stem):
let interrog = v.shape_a(VerbTense::Past, VerbPerson::Base);  // "ar mhol"
let negative = v.shape_n(VerbTense::Past, VerbPerson::Base);  // "níor mhol"
```

The `Verb` struct fields:
- `past`, `pres_cont`, `fut`, `cond`, `past_hab`, `pres_subj` — each a
  `TenseForms { indep, dep, rel_indep }` of `PersonForms { base, sg1..sg3, pl1..pl3, auto }`
- `imper` — `PersonForms` (no independent/dependent split)
- `verbal_noun: Vec<Form>`, `verbal_adjective: Vec<Form>`

For irregular verbs, pass `VerbConjugationClass::Irregular` — `from_lemma` returns
an empty `Verb` (irregulars are loaded from BuNaMo XML via `Verb::from_xml`).

### Adjectives

```rust
use gramadan::adjective::{self, Adjective, Declension};

let dec = adjective::guess_declension("mór");  // → First
let adj = Adjective::from_lemma("mór", dec);

// Access forms — each is Option<String>:
adj.sg_nom;        // "mór"
adj.sg_gen_masc;   // "mhóir" (after masc noun)
adj.sg_gen_fem;    // "móire" (after fem noun)
adj.pl_nom;        // "móra"
adj.graded;        // "mó" (comparative/superlative)
```

## WASM binding (gramadan-wasm)

The binding is **already wired** in `gramadan-wasm/src/lib.rs`. Three functions
produce `Paradigm { declension, supported, forms }`:

- `noun_paradigm(lemma, gender, class)` — existing
- `verb_paradigm(lemma, class)` — emits VN, VA, all tenses, shape rules, imperative
- `adjective_paradigm(lemma, class)` — emits sg_nom, sg_gen (masc/fem), pl_nom, graded

Each is called from the exported `generate(lemma, pos, class, gender)` dispatcher.

### Tag vocabulary (unchanged from BuNaMo)

The WASM forms use these tags, consumed by Gréasán's
`app/src/lib/paradigm.ts::buildParadigm(forms, pos)`:

| Slot | Tags |
|---|---|
| verbal noun | `["verbal-noun"]` |
| verbal adjective | `["verbal-adjective"]` |
| past 1sg independent | `["past", "indicative", "first-person", "singular"]` |
| past 1sg dependent | `["past", "indicative", "first-person", "singular", "dependent"]` |
| past interrogative (dep-a) | `["past", "indicative", ..person, "dep-a"]` |
| past negative (dep-n) | `["past", "indicative", ..person, "dep-n"]` |
| imperative 2sg | `["imperative", "second-person", "singular"]` |
| adj sg gen masc | `["genitive", "singular", "masculine"]` |
| adj graded | `["comparative"]` |

The frontend `VerbParadigm` and `AdjParadigm` render types already consume these
tags — **no Gréasán frontend change is required**.

## Python / PyO3 path (pipeline enrichment)

The Gréasán build pipeline (`src/goidelic/grammar.py`) currently calls
`gramadan_rs.enrich(word, pos, gender, "")` to get `(grammar_class, method)`.
This uses the PyO3 binding in `gramadan-rs/src/lib.rs` (behind the `python`
feature flag, built by maturin).

### Adding VN/VA to the pipeline

To enrich Téarma verb entries with verbal noun and verbal adjective at build
time, the pipeline needs a new PyO3 function. The Rust side:

```rust
// In gramadan-rs/src/lib.rs, under #[cfg(feature = "python")]
#[pyfunction]
fn verb_principal_parts(lemma: &str, class: &str) -> (String, String) {
    let vc = match class {
        "1" => VerbConjugationClass::First,
        "2" => VerbConjugationClass::Second,
        _ => return (String::new(), String::new()),
    };
    let v = Verb::from_lemma(lemma, vc);
    let vn = v.verbal_noun.first().map(|f| f.value.clone()).unwrap_or_default();
    let va = v.verbal_adjective.first().map(|f| f.value.clone()).unwrap_or_default();
    (vn, va)
}
```

The Python side (`src/goidelic/grammar.py`):

```python
def enrich_verb_forms(word: str, grammar_class: str) -> tuple[str, str]:
    """Return (verbal_noun, verbal_adjective) for a verb lemma.

    Requires the grammar_class ("1" or "2") to have been resolved already
    (via enrich_grammar_class). Irregular verbs return empty — their forms
    come from BuNaMo XML, not generation.
    """
    if _gramadan is None or grammar_class not in ("1", "2"):
        return "", ""
    return _gramadan.verb_principal_parts(word, grammar_class)
```

### Integration into the Gréasán build

The verb entry enrichment in `scripts/build-bunamo-data.py` (or wherever Téarma
verb records are processed) would call:

```python
grammar_class, confidence = enrich_grammar_class(lemma, "verb", "")
vn, va = enrich_verb_forms(lemma, grammar_class)
# Embed vn/va into the entry's data alongside grammar_class
```

### Attestation / confidence

The `grammar_class_confidence` field already flows through for verbs (driven by
`Method::VerbHeuristic` → "uncertain" in `METHOD_CONFIDENCE`). When the class
is guessed, the generated VN/VA inherits that uncertainty — Gréasán shows a `?`
via `grammarClassConfidence`. No additional confidence plumbing is needed for
VN/VA specifically, since the class determines the forms.

If BuNaMo attests the verb (the verb exists in `bunamo_lexical_entry_data.csv`),
the class is "attested" and the VN/VA are reliable. If the class was guessed,
the VN/VA are only as good as the guess — but for regular verbs (which is all
the guesser handles), the VN/VA rules are deterministic from the class.

## Remaining work

### Noun full paradigm (not yet exposed)

Nouns currently expose only nominative + genitive via the WASM binding.
The singular sub-paradigm (vocative, dative) is computed internally by
`generate_genitive_*` but only the genitive is returned. A `singular_paradigm`
function returning the full `SingularInfo` would complete the noun table. See
the "noun.rs" section of the original development notes for the refactor plan.

### Report shape (g = subordinate)

`shape_a` (interrogative) and `shape_n` (negative) are implemented.
The report/subordinate shape (`go/gur`) is trivially derivable: `an→go`,
`ar→gur`, same mutation. Not computed or stored — the frontend can derive it
from `dep-a` by swapping the particle prefix. This was a deliberate decision
(see HANDOFF-verb-shape-rules.md).
