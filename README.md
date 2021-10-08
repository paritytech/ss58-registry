# SS58 Registry

[![GitHub license](https://img.shields.io/badge/license-Apache2-green)](#LICENSE) [![GitLab Status](https://gitlab.parity.io/parity/ss58-registry/badges/main/pipeline.svg)](https://gitlab.parity.io/parity/ss58-registry/pipelines)

A list of known [SS58](https://github.com/paritytech/substrate/wiki/External-Address-Format-(SS58)) account types as an enum.

This is driven from the [json data file](ss58-registry.json) which contains entries like this:

```js
{
	"prefix": 5,                      // unique u16
	"network": "plasm",               // unique no spaces
	"displayName": "Plasm Network",   //
	"symbols": ["PLM"],               // symbol for each ballance pallet (usually one)
	"decimals": [15],                 // decimals for each symbol listed.
	"standardAccount": "*25519",      // Sr25519, Ed25519 or secp256k1
	"website": "https://plasmnet.io"  // website or github of network
},
```

(Typically used by the Polkadot, Kusama or Substrate ecosystems.)

## Process:

1. Fork and clone this repo

2. Add an additional account type to `ss58-registry.json` (contiguous prefixes are better)

3. Bump the minor (middle) version number of the `Cargo.toml` by running:
```
cargo install cargo-bump && cargo bump minor
```
4. git stage, commit, push and then raise a pull request

5. Once the PR has landed, one of the admins can
[create a new release](https://github.com/paritytech/ss58-registry/releases/new).
This will release the new version to [crates.io](https://crates.io/crates/ss58-registry)

## Licensing:

Apache-2.0
