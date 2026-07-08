# MAP Deploy-Review Contract

This crate exposes the deterministic MAP manifest review contract for
`apiVersion: map.mithran/v1`.

`map-cli` can depend on the public mirror at
`https://github.com/mithran-hq/map-deploy-review-contract` without requiring
access to the private `mithran-control-plane` source repository. The private
control-plane repository owns the contract source and mirrors this crate there
when the contract changes.

## API

Call:

```rust
map_deploy_review_contract::review_manifest(manifest_yaml, "mithran.yaml")
```

The response schema is `map-deploy-review-contract/v1` and includes:

- `apiVersion: map.mithran/v1`
- `status`: `passed` or `blocked`
- `findings`: hard `ERR_*` findings with `level`, `code`, `message`, and `path`
- `finding_codes`: ordered `ERR_*` codes for scripts
- `normalized_summary`: redacted normalized manifest summary on pass

The contract does not call the live deploy-review service, upload source,
create deployment state, mutate routes, or mint evidence.

## Fixtures

`tests/fixtures/map-deploy-review-contract/cases.yml` is the parity matrix.
`cargo test` verifies the crate against that matrix.
