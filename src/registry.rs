use super::*;

include!(concat!(env!("OUT_DIR"), "/registry_gen.rs"));

#[cfg(feature = "std")]
impl std::fmt::Display for Ss58AddressFormatRegistry {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let lookup = PREFIX_TO_INDEX
			.binary_search_by_key(&from_known_address_format(*self), |(prefix, _)| *prefix)
			.expect("always be found");
		let (_, idx) = PREFIX_TO_INDEX[lookup];
		write!(f, "{}", ALL_SS58_ADDRESS_FORMAT_NAMES[idx])
	}
}

impl TryFrom<Ss58AddressFormat> for Ss58AddressFormatRegistry {
	type Error = ParseError;

	fn try_from(x: Ss58AddressFormat) -> Result<Ss58AddressFormatRegistry, ParseError> {
		PREFIX_TO_INDEX
			.binary_search_by_key(&x.prefix(), |(prefix, _)| *prefix)
			.map(|lookup| {
				let (_, idx) = PREFIX_TO_INDEX[lookup];
				ALL_SS58_ADDRESS_FORMATS[idx]
			})
			.map_err(|_| ParseError)
	}
}

/// const function to convert [`Ss58AddressFormat`] to u16
pub const fn from_known_address_format(x: Ss58AddressFormatRegistry) -> u16 {
	x as u16
}

#[cfg(feature = "std")]
impl std::fmt::Debug for TokenRegistry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let (name, decimals) = self.attributes();
		f.debug_struct("TokenRegistry")
			.field("name", &name)
			.field("decimals", &decimals)
			.finish()
	}
}

impl TokenRegistry {
	/// Creates the specified amount of [`Token`] with its name and decimals filled from the
	/// [`TokenRegistry`] variant.
	///
	/// ```
	/// # use ss58_registry::TokenRegistry;
	/// # #[cfg(feature = "std")]
	/// # fn x() {
	/// let my_token = TokenRegistry::Dot.create_token(100_000_000);
	/// assert_eq!(format!("{}", my_token), "0,010 DOT");
	/// assert_eq!(format!("{:?}", my_token), "0,010 DOT (100_000_000)");
	/// # }
	/// # #[cfg(not(feature = "std"))]
	/// # fn x() {}
	/// # x();
	/// ```
	pub fn create_token(&self, amount: u128) -> Token {
		let (name, decimals) = self.attributes();
		Token { name, decimals, amount }
	}
}
