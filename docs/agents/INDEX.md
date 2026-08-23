---
title: MAP deploy-review contract agent corpus index
answers: Where does an agent enter the contract corpus, and which documents answer each context question?
last_verified: 2026-08-23
verified_ref: f81ea3cbd02de28536b7b095fce41ec7db324113
refresh: quarterly
---

# Agent corpus index

Start with this document before reconstructing repository context.

```claim
id: corpus.entry-path
kind: MEASURED
statement: The corpus entry path is docs/agents/INDEX.md.
command: test -f docs/agents/INDEX.md && printf '%s\n' docs/agents/INDEX.md
expect: "docs/agents/INDEX.md"
match: exact
verified_at: 2026-08-23
verified_ref: f81ea3cbd02de28536b7b095fce41ec7db324113
```

## Freshness

| Document | Last verified |
| --- | --- |
| `INDEX.md` | 2026-08-23 |
| `CORPUS-SPEC.md` | 2026-08-12 |
| `current-state/repository-shape.md` | 2026-08-23 |
| `hazards/private-source-public-mirror.md` | 2026-08-23 |
| `verification/REFRESH.md` | 2026-08-23 |

```claim
id: corpus.estate-directory
kind: INFERRED
statement: The estate directory is mithran-hq/mithran-infra/docs/agents/current-state/estate-corpus-directory.md.
basis: The estate task names that landed path as the cross-repository ownership index.
falsifier: The named mithran-infra path routes this repository elsewhere.
verified_at: 2026-08-23
verified_ref: f81ea3cbd02de28536b7b095fce41ec7db324113
```

| Document | Answers |
|---|---|
| `current-state/repository-shape.md` | What public Rust API and fixture surface does the contract expose? |
| `hazards/private-source-public-mirror.md` | Which source and mirror boundary can drift? |
| `verification/REFRESH.md` | Which claims can an agent refresh manually? |
| `CORPUS-SPEC.md` | What evidence grammar applies to these documents? |

```claim
id: corpus.inventory
kind: MEASURED
statement: The corpus contains exactly the five listed Markdown documents.
command: find docs/agents -type f -name '*.md' | sort | paste -sd' ' -
expect: "docs/agents/CORPUS-SPEC.md docs/agents/INDEX.md docs/agents/current-state/repository-shape.md docs/agents/hazards/private-source-public-mirror.md docs/agents/verification/REFRESH.md"
match: exact
verified_at: 2026-08-23
verified_ref: f81ea3cbd02de28536b7b095fce41ec7db324113
```

```claim
id: corpus.index-lists-every-document
kind: MEASURED
statement: The index's non-claim tables name every Markdown document in this corpus.
command: |
  table="$(awk 'BEGIN {f=0} /^(```|~~~)/ {f=!f; next} !f && (/^[|]/ || /^- `/) {print}' docs/agents/INDEX.md)"
  missing=0
  while IFS= read -r path; do rel="${path#docs/agents/}"; if printf '%s\n' "$table" | grep -Fq "\`$rel\`"; then :; elif printf '%s\n' "$table" | awk -v p="$rel" -F'|' '$0 ~ /^\\|/ {for (i=1; i<=NF; i++) {gsub(/^[ \\t]+|[ \\t]+$/, "", $i); if ($i == p) found=1}} END {exit !found}'; then :; else missing=1; fi; done < <(find docs/agents -type f -name '*.md' | sort)
  test "$missing" -eq 0 && printf '%s\n' complete
expect: "complete"
match: exact
drift: alert
verified_at: 2026-08-23
verified_ref: f81ea3cbd02de28536b7b095fce41ec7db324113
```
