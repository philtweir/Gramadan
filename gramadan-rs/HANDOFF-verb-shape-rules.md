# Handoff — verb shape rules (interrog / negative / subordinate)

**Status: COMPLETE.** The shape-rule subsystem is built, tested, and wired into
the WASM binding. Gréasán can consume `dep-a` (interrogative positive) and
`dep-n` (declarative negative) tagged forms directly.

## What's built

### Rust API (`src/verb.rs`)

Two methods on `Verb`:

```rust
// Interrogative positive: "ar mhol", "an ndéanann", etc.
v.shape_a(VerbTense::Past, VerbPerson::Base) -> Option<String>

// Declarative negative: "níor mhol", "ní dhéanann", etc.
v.shape_n(VerbTense::Past, VerbPerson::Base) -> Option<String>
```

Each calls the dependent stem, applies the rule's mutation via `opers::mutate`,
and prepends the particle. Returns `None` only for irregular verbs whose
dependent forms aren't available (irregulars loaded from XML have full forms).

Internally driven by two match-based functions (`interrog_pos_rule`,
`declar_neg_rule`) returning `(&str, Mutation)` — particle + mutation — for
each lemma × tense × person combination. All 9 irregular verb overrides
(bí, abair, déan, faigh, feic, téigh, tar, clois, cluin) are handled.

### WASM binding (`gramadan-wasm/src/lib.rs`)

`push_shaped()` emits `FormItem`s for all 4 tenses (past/present/future/
conditional) × 8 persons (base, sg1–sg3, pl1–pl3, auto), tagged:

- `dep-a`: `[tense, "indicative", ...person_tags, "dep-a"]`
- `dep-n`: `[tense, "indicative", ...person_tags, "dep-n"]`

Two WASM-level tests (`shaped_forms_are_emitted`, `independent_forms_are_realised`)
verify the output.

### Cross-validation

`src/bin/test_shapes.rs` — 30 checks covering 2 regular verbs (mol, oscail)
× 5 tense/person combos + 9 irregular verbs from XML. All pass.

## Report shape (g = subordinate)

**Not ported — by design.** `go`/`gur` use the same mutation as `an`/`ar`
(interrogative), so `g` is trivially derived from `dep-a`:
- `an` → `go` (both eclipse)
- `ar` → `gur` (both lenite)

The frontend can swap the particle prefix if it wants to render the subordinate
column. This holds for all irregulars too — the eclipsis-vs-lenition split
between interrogative and subordinate is shared.

## Gréasán integration

### Live generation (WASM, already wired)

The WASM `verb_paradigm()` function already calls `push_shaped()`. Any entry
page that renders a verb paradigm via `gramadan-wasm` gets `dep-a`/`dep-n`
forms automatically. The frontend's `VerbParadigm` render type needs to
handle the `dep-a`/`dep-n` tags to show the dependent view columns.

The three-column dependent view maps to:
- **a** (interrogative) = forms tagged `dep-a`
- **n** (negative) = forms tagged `dep-n`
- **g** (subordinate) = derive from `dep-a` by replacing `an `→`go `, `ar `→`gur `

### Baked forms (build-time, BuNaMo pipeline)

If the build pipeline (`scripts/build-bunamo-data.py`) wants to bake
dependent forms into the static data, it can call `Verb::from_lemma` via
PyO3 and extract `shape_a`/`shape_n` for each tense × person. This replaces
the hand-rolled `depParticle`/`mutateForParticle` logic in
`app/src/views/EntryDetail.svelte` (which is irregular-buggy).

## Decisions (do not relitigate)

- Report (g) NOT ported — trivially derivable from a (an→go, ar→gur, same mutation)
- No separate rule-table struct — two match-based functions, exhaustively tested
- v1 py stays upstream-clean; all py changes live in v2
- gramadan-py now sourced from `../Gramadan/Python` (pyproject/uv.lock in Gréasán);
  the old `Gréasán/data/gramadan-src` vendored copy was removed
