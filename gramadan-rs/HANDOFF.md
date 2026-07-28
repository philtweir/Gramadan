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

**Full Téarma pipeline estimated accuracy: 99.7%** when BuNaMo is pre-loaded.
Only ~200 words (0.4%) hit the morphological guesser; the rest are covered by
records, stated classes, or high-confidence heuristics.

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

### 1b. Build LemmaDb from BuNaMo

BuNaMo is loaded as a separate layer (`bunamo-v2`). At the point where
Téarma is built, BuNaMo may or may not already be available on-device.

**Option A (preferred):** Ship a pre-compiled LemmaDb as a small binary blob
(~200KB for 12k entries) inside the core bundle. Generate it from BuNaMo XML
at CI time. Load it in `builder_plugin.rs` before the TBX parse.

**Option B:** If BuNaMo layer is guaranteed to be loaded first, read its
`head.sqlite` to build the LemmaDb at runtime. The spine + concept_tags
tables have what's needed.

**Option C (minimum viable):** Pass an empty `LemmaDb::new()`. The guesser
still works at 92% from morphological rules alone; compound decomposition
just won't fire. This is a valid starting point.

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

Two options:

**Option A (call Rust from Python via PyO3):** Build gramadan-rs as a Python
extension module. This keeps one implementation. Requires adding PyO3
bindings — not done yet but straightforward given the simple API surface.

**Option B (pure Python):** The Python v2 code in `gramadan/v2/` already has
the guesser classes. Call `NualeargaisNounDeclensionGuesser` directly in the
pipeline. This won't have the compound decomposition improvements but covers
the basics. The heuristics (vowel→4th, VN→1st) can be added as a few lines
of Python.

Option A is cleaner long-term; Option B is faster to ship.

---

## 3. Scope guardrails

- **BuNaMo first.** Where a lemma exists in BuNaMo, its grammar_class is
  exact — use the LemmaDb lookup, don't guess.
- **Don't override stated classes.** If Téarma already has `grammar_class`
  from `extract_declension()`, keep it.
- **Don't force a class on plural-only entries** (`iol`/`pl`/`fir iol`),
  abbreviations (`gior`/`abr`), or collectives (`cnuas`). The handoff from
  the previous session has the breakdown (16.3% of Téarma nouns lack a class;
  only ~10% are the recoverable bare `fir`/`bain` gap).
- **Verbs:** Téarma tags verbs only `br`/`v` (no class). The lemma-only
  heuristic is 94.6% accurate. Ship it.
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

- **PyO3/WASM bindings** for gramadan-rs — needed for Python path (Option A)
  and for browser-side on-demand declension in the word view.
- **Fix Kaikki gender pipeline loss** — 14k nouns have gender in the Kaikki
  dump but it's dropped before reaching the graph. `normalise.py` maps the
  tags correctly; the issue is likely in how the graph import handles
  entry-level vs form-level features.
- **Pre-compiled LemmaDb blob** — generate at CI, ship in core bundle, avoid
  needing BuNaMo layer loaded before Téarma build.
- **Adjective declension** — not implemented in gramadan-rs yet. Adjectives
  reuse `SingularInfo` classes (simpler than nouns). Téarma has `a1`..`a3`.
- **On-demand paradigm generation** — the `singular_info` and `plural_info`
  modules can generate full paradigms given (lemma, gender, declension), at
  98.54% accuracy. Wire into the word view for live declension tables.
