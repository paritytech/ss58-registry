use super::{Ss58AddressFormat, Ss58AddressFormatRegistry};
use std::convert::TryInto;

#[test]
fn is_reserved() {
	let reserved: Ss58AddressFormat = Ss58AddressFormatRegistry::Reserved46Account.into();
	assert!(reserved.is_reserved());

	let not_reserved: Ss58AddressFormat = Ss58AddressFormatRegistry::PolkadexAccount.into();
	assert!(!not_reserved.is_reserved());

	assert!(!Ss58AddressFormat::custom(100).is_reserved());
}

#[test]
fn is_custom() {
	assert!(Ss58AddressFormat::custom(432).is_custom());

	let reserved: Ss58AddressFormat = Ss58AddressFormatRegistry::Reserved46Account.into();
	assert!(!reserved.is_custom());

	let not_reserved: Ss58AddressFormat = Ss58AddressFormatRegistry::PolkadexAccount.into();
	assert!(!not_reserved.is_custom());
}

#[test]
fn enum_to_name_and_back() {
	for name in Ss58AddressFormat::all_names() {
		let val: Ss58AddressFormatRegistry = (*name).try_into().expect(name);
		assert_eq!(name, &val.to_string());
	}
}

#[test]
fn blah() {
	let dot: Ss58AddressFormat = Ss58AddressFormatRegistry::PolkadotAccount.into();
	//dot.tokens();
	assert_eq!(dot.prefix(), 0);
	let ksm: Ss58AddressFormat = Ss58AddressFormatRegistry::KusamaAccount.into();
	assert_eq!(ksm.prefix(), 2);
}
