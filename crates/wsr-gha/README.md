# GitHub Actions provider

This crate implements the bounded built-in static inspection frontend, with CLI acceptance
and fixture/reference tests. See the [supported subset](../../docs/cli/README.md#static-workflow-inspection).
The canonical [GHA provider guide](../../docs/provider/gha/README.md) owns architecture,
compatibility, state and validation contracts under the central documentation tree.
See the [crate inventory](../README.md) for source and publication status.

[Frontend modeling references](docs/frontend-modeling.md) catalogs concrete upstream enums,
structs and validation patterns for a WSR-owned Rust frontend. It is a design proposal, not a
complete schema, dependency-adoption decision or implementation claim.
