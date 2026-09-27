# Verb XML patches

Compile-time additions to the irregular-verb XML in the BuNaMo `data/` submodule,
applied by `../build.rs`. Upstream BuNaMo omits some forms gramadan needs (e.g.
`bí`'s RelIndep relative forms *a bhíos / a bheas*); rather than fork BuNaMo or
edit `src/`, each file here is an XML fragment that `build.rs` injects into the
matching `data/verb/<same-name>` just before its closing `</verb>` (the parser
collects `<tenseForm>`/`<moodForm>` by attributes, so order is irrelevant).

To patch another verb: drop `patches/<verb>_verb.xml` here containing the extra
`<tenseForm .../>` (or `<moodForm .../>`) lines. Application is idempotent — the
first non-empty line of the fragment is the "already applied" marker — so a build
against a `data/` that already has the forms is a no-op.
