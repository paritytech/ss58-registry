use super::*;

include!(concat!(env!("OUT_DIR"), "/account_type_enum.rs"));

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
