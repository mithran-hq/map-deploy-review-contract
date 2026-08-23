---
title: MAP deploy-review contract repository shape
answers: What public Rust API, schema, and fixture surface does this repository expose?
last_verified: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
refresh: quarterly
---

# Repository shape

This repository is a small Rust crate. It exposes deterministic manifest review to MAP clients.

```claim
id: repository.tracked-surface
kind: MEASURED
statement: The repository tracks Cargo.toml, README.md, src/lib.rs, and the contract fixture matrix.
command: git ls-files Cargo.toml README.md src/lib.rs tests/fixtures/map-deploy-review-contract/cases.yml | sort | paste -sd' ' -
expect: "Cargo.toml README.md src/lib.rs tests/fixtures/map-deploy-review-contract/cases.yml"
match: exact
verified_at: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
```

```claim
id: repository.public-api
kind: MEASURED
statement: The crate exposes one public review_manifest function in src/lib.rs.
command: grep -Fxc 'pub fn review_manifest(' src/lib.rs
expect: "1"
match: exact
verified_at: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
```

```claim
id: repository.response-schema
kind: MEASURED
statement: The implementation contains the API version, error-code, and normalized-summary response fields.
command: |
  for p in apiVersion ERR_API_VERSION normalized_summary; do grep -Fq "$p" src/lib.rs || exit 1; done
  printf '%s\n' response-fields-present
expect: response-fields-present
match: exact
verified_at: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
```

```claim
id: repository.fixture-matrix
kind: MEASURED
statement: The repository tracks the deploy-review parity fixture matrix.
command: git ls-files tests/fixtures/map-deploy-review-contract/cases.yml
expect: "tests/fixtures/map-deploy-review-contract/cases.yml"
match: exact
verified_at: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
```

```claim
id: repository.contract-version
kind: ASSERTED
statement: The README names map-deploy-review-contract/v1 as the response schema.
source: README.md
quote: "The response schema is map-deploy-review-contract/v1 and includes:"
verified_at: 2026-08-23
verified_ref: c7fc08161013f665888b1f2490807256cf96fda0
```
