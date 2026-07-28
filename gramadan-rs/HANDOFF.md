# Hand-off — gramadan-rs integration into Gréasán

For a **separate session**. This session built the engine and validated it;
the next wires it into the Gréasán app so every Téarma noun gets a
grammar_class at build time.

---

## 0. What is already done (do NOT redo)

**gramadan-rs** (`gramadan-rs/`) — a self-contained Rust crate implementing
Irish morphology: mutation, slenderization, broadening, syncope, genitive
generation, noun declension guessing, verb conjugation inference, and a
compound decomposition engine.

**Public API** — `enrich::enrich_grammar_class(&Record, &LemmaDb) -> EnrichResult`
cascades through 6 strategies:

1. Already stated → keep
2. LemmaDb lookup (BuNaMo/Kaikki exact match)
3. Heuristics: vowel-ending→4th, `-adh`/`-eadh`→1st, `-áil`→2nd, proper noun→4th
4. Compound decomposition (lenition detection, hyphen split, direct suffix ≥7 chars)
5. Morphological guesser (gender + ending patterns)
6. Verb: lemma-only heuristic (monosyllabic→1st, `-igh`→2nd, irregulars list)

Each result carries a `Method` enum so the caller can audit/log what resolved it.

**Cross-validated** against three independent corpora:

| Dataset | Compound guesser | Notes |
|---------|-----------------|-------|
| BuNaMo (12,377 nouns) | 93.14% | Training data |
| Téarma single-word (33,076) | 89.39% | Stated class as ground truth |
| Kaikki/Wiktionary novel (3,226) | 87.91% | Genitive-derived ground truth |
| Verbs lemma-only (3,359) | 94.55% | |
| Verbs future-based (3,359) | 100% | |

**Two metrics — keep them apart (the earlier "99.7%" conflated them).** The
cross-validation table above is *engine accuracy on specific sets*, NOT pipeline
coverage. Measured on the real Téarma corpus at wiring time (Gréasán session):

- **Pipeline accuracy** over all Téarma nouns is high but *dominated by the ~33k
  that already have a stated class* — it says almost nothing about the enrichment.
- **Enrichment on the CLASSLESS subset** (the metric that actually matters):
  - **~29,254 classless** entries — 22,884 nouns + 6,370 verbs (Téarma never
    tags verb class).
  - Guesser resolves **79.7%** (empty DB) → **79.8%** with BuNaMo preloaded. The
    ~20% Unresolved are no-gender, multi-word, or plural-only entries. Resolved
    accuracy is ~89% (per the table).

**BuNaMo preload is a CONFIDENCE upgrade, not a coverage or (naïve) accuracy
win.** It moves coverage by **+18**. Of the ~1,104 overlap entries (BuNaMo
attests AND the guesser also answered) the two disagree **92.6%** — but that is
NOT 92.6% of guesses being wrong. Almost every disagreement is BuNaMo returning
**declension 0** (which the binding was mapping to `"irr"`) on **verbal nouns**,
against the guesser's sensible `-ú → 4th` / `-adh → 1st`:

- Genuine BuNaMo wins are **common nouns** (`bealach`→1, `carraig`→2) — ~1,122
  exact `DbLookup` hits → *attested* confidence.
- The "92.6% correction" is a **declension-0 / verbal-noun modelling clash**, not
  real corrections; mapping `0 → irr` is misleading (a productive `-ú` verbal
  noun is not irregular like `bean`).

**Declension-0 handling — fix before trusting the preload.** Don't let BuNaMo's
`declension 0` override the verbal-noun heuristics. `-adh/-eadh → 1st` is
validated at **98.1%** (104/106 BuNaMo `-adh/-eadh` nouns are 1st; the two
exceptions `conradh`/`feadh` are standalone, already in the irregular list).
Options: (a) skip `DbLookup` when BuNaMo=0, fall back to the heuristic; (b) keep
0 but label it truthfully (verbal-noun / no-class), not `"irr"`; (c) trust
BuNaMo only for declensions 1–5.

**Key data findings from this session:**

- Téarma classless vowel-ending nouns: 100% are 4th declension (tested on
  BuNaMo overlap subset, n=80)
- Téarma classless consonant-ending: 77% are verbal nouns (`-adh`/`-eadh`),
  reliably 1st declension; remainder are mostly proper nouns (→4th) or
  compounds the guesser handles
- The irreducible error floor is ~7%, caused by 2↔3 and 3↔1 ambiguity on
  common short words (`cos`, `lámh`, `beart`, `luach`) — no surface signal
  distinguishes these without a known genitive
- Wiktionary's Kaikki dump has 18.9k nouns with gender but only 4.2k make it
  through to the Gréasán graph — the pipeline drops gender for ~14k entries
  (a separate fix, not blocking)

---

## 1. Wire gramadan-rs into Gréasán Rust path

The on-device TBX build in `builder_plugin.rs` (line ~417) calls
`tbx_parser::parse_tbx()` → `records_to_csv()`. The enrichment step goes
between them.

### 1a. Add dependency

In `Gréasán/app/src-tauri/Cargo.toml`:

```toml
[dependencies]
gramadan = { path = "../../../Gramadan/gramadan-rs" }
```

### 1b. LemmaDb — optional, not required

The `enrich_grammar_class()` API takes a `&LemmaDb` for compound
decomposition (matching compound heads against known words). However,
analysis showed that BuNaMo/Kaikki lookup only corrects **16 words** on
the classless Téarma subset — and all 16 are now baked into the
`FULLY_IRREGULAR` exception list in `noun.rs`. So the LemmaDb is
**not required for accuracy**.

**Pass `LemmaDb::new()` (empty).** The guesser works at 93%+ from
morphological rules + exception lists alone. Compound decomposition
(which needs the LemmaDb) adds ~1% on BuNaMo but its value on Téarma
is marginal since most Téarma compounds are verbal nouns handled by
the `-adh→1st` / `-ú→4th` heuristics.

If you later want compound decomposition for non-Téarma use cases (e.g.
user-entered words), populate the LemmaDb from BuNaMo at layer load time.

### 1c. Enrich records

In `builder_plugin.rs`, after `parse_tbx()` returns records and before
`records_to_csv()`:

```rust
use gramadan::enrich::{self, Record as EnrichRecord};
use gramadan::noun::LemmaDb;

// ... load db (see 1b) ...

for rec in &mut records {
    if rec.grammar_class.is_empty() && (rec.pos == "noun" || rec.pos == "verb") {
        let gender = rec.forms.first()
            .and_then(|f| f.gram_features.first())
            .cloned()
            .unwrap_or_default();
        let result = enrich::enrich_grammar_class(
            &EnrichRecord {
                word: rec.word.clone(),
                pos: rec.pos.clone(),
                gender,
                grammar_class: String::new(),
            },
            &db,
        );
        rec.grammar_class = result.grammar_class;
    }
}
```

### 1d. Verify

The cross-validation binary (`gramadan-rs/src/bin/cross_validate.rs`) has
three modes:

```bash
# BuNaMo validation (default)
cargo run --release --bin cross-validate -- path/to/data

# Téarma validation
cargo run --release --bin cross-validate -- --tearma tearma.tsv path/to/data

# Kaikki/Wiktionary validation
cargo run --release --bin cross-validate -- --kaikki kaikki.tsv path/to/data
```

The `scripts/python_guessers.py` script runs the Python guessers for
comparison. The dump mode (`--dump-guesses`) outputs TSV for diffing.

---

## 2. Wire into Python build path (pre-built layers)

The Python path in `Gréasán/src/goidelic/run.py` runs `run_tbx_pipeline()`:
tbx.py → ontolex.py → arches.py. The enrichment step goes between tbx and
ontolex.

**PyO3 bindings exist** (`src/python.rs`, gated behind `features = ["python"]`).
The Cargo.toml already has `crate-type = ["rlib", "cdylib"]` and optional
`pyo3` dependency. Build with maturin:

```bash
cd gramadan-rs && maturin develop --features python
```

Then in the Python pipeline:

```python
import gramadan_rs
result = gramadan_rs.enrich(word, pos, gender, grammar_class)
# result.grammar_class, result.method
```

An empty LemmaDb is sufficient (see §1b). The 16 exception words are baked
into the Rust code; no external data needed.

---

## 3. Scope guardrails

- **Don't override stated classes.** If Téarma already has `grammar_class`
  from `extract_declension()`, keep it. The `enrich_grammar_class` API
  already handles this (returns `AlreadyStated` when `grammar_class` is
  non-empty).
- **Don't force a class on plural-only entries** (`iol`/`pl`/`fir iol`),
  abbreviations (`gior`/`abr`), or collectives (`cnuas`). The handoff from
  the previous session has the breakdown (16.3% of Téarma nouns lack a class;
  only ~10% are the recoverable bare `fir`/`bain` gap).
- **Verbs:** Téarma tags verbs only `br`/`v` (no class). The lemma-only
  heuristic is 94.6% accurate. Ship it.
- **BuNaMo LemmaDb is optional.** The 16 words where BuNaMo/Kaikki would
  have corrected the guesser are now in the `FULLY_IRREGULAR` exception
  list. An empty `LemmaDb::new()` is sufficient for the Téarma pipeline.
- **BuNaMo declension 0.** If you do load BuNaMo into the LemmaDb, skip
  entries with `declension=0`. These are mostly verbal nouns that BuNaMo
  didn't classify; the heuristics (`-adh→1st`, `-ú→4th`) are more useful
  than an `"irr"` label. Trust BuNaMo only for declensions 1–5.
- **`Method` audit trail:** Log or store which strategy resolved each word.
  The ~200 words hitting `MorphologicalGuesser` are the ones most likely to
  be wrong.

---

## 4. Key files

| Thing | Path |
|---|---|
| gramadan-rs crate | `Gramadan/gramadan-rs/` |
| Enrichment API | `gramadan-rs/src/enrich.rs` |
| LemmaDb + compound decomposition | `gramadan-rs/src/noun.rs` |
| Morphological operations | `gramadan-rs/src/opers.rs` |
| Singular genitive strategies | `gramadan-rs/src/singular_info.rs` |
| Verb conjugation inference | `gramadan-rs/src/verb.rs` |
| Cross-validation binary | `gramadan-rs/src/bin/cross_validate.rs` |
| Python comparison script | `gramadan-rs/scripts/python_guessers.py` |
| Gréasán TBX parser (injection point) | `Gréasán/app/src-tauri/src/tbx_parser.rs` ~line 511 |
| Gréasán builder (orchestration) | `Gréasán/app/src-tauri/src/builder_plugin.rs` ~line 417 |
| Python pipeline (injection point) | `Gréasán/src/goidelic/run.py` `run_tbx_pipeline()` |
| BuNaMo data generator | `Gréasán/scripts/build-bunamo-data.py` |
| Previous handoff (grammar-class scope) | `Gréasán/HANDOFF-grammar-class-inference.md` |

## 5. Follow-ups (not blocking)

- **PyO3 bindings exist** (`src/python.rs`) but may need extending if the
  Gréasán Python pipeline wants batch mode or LemmaDb population from Python.
- **WASM bindings** — not yet built. Needed for browser-side on-demand
  declension in the word view.
- **Fix Kaikki gender pipeline loss** — 14k nouns have gender in the Kaikki
  dump but it's dropped before reaching the Gréasán graph. `normalise.py`
  maps the tags correctly; the issue is likely in how the graph import
  handles entry-level vs form-level features. Not blocking for Téarma
  enrichment but would improve Wiktionary layer quality.
- **Adjective declension** — not implemented in gramadan-rs yet. Adjectives
  reuse `SingularInfo` classes (simpler than nouns). Téarma has `a1`..`a3`.
- **On-demand paradigm generation** — the `singular_info` and `plural_info`
  modules can generate full paradigms given (lemma, gender, declension), at
  98.54% accuracy. Wire into the word view for live declension tables.
