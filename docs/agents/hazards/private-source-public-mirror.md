---
title: MAP deploy-review source and mirror hazard
answers: Which source and mirror boundary can drift?
last_verified: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
refresh: quarterly
---

# Source and mirror hazard

The public crate is a consumer-facing mirror. The private control-plane repository owns the source contract.

## Observed correction

During the 2026-08-23 corpus review, the response-schema evidence was narrowed to
the exact fields present in the source and public mirror. The correction prevents
a local mirror shape from being treated as proof of private-source parity.

```claim
id: hazard.source-owner
kind: ASSERTED
statement: The private control-plane repository owns the contract source and mirrors this crate when the contract changes.
source: README.md
quote: "The private control-plane repository owns the contract source and mirrors this crate there when the contract changes."
verified_at: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
```

```claim
id: hazard.public-mirror-path
kind: MEASURED
statement: The public mirror is referenced by the repository README.
command: grep -F -l 'github.com/mithran-hq/map-deploy-review-contract' README.md
expect: "README.md"
match: exact
verified_at: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
```

Do not treat a local change as authoritative until the private source and mirror parity path are reconciled.
