# SS58 Registry

List of wellknown [SS58](https://github.com/paritytech/substrate/wiki/External-Address-Format-(SS58)) account types as an enum.

This is driven from the [json data file](src/ss58-registry.json) which contains entries like this:

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

If you wish to add an additional account type then please raise a pull request adding to the json file.

## Licensing:

Apache-2.0
