use super::{Ss58AddressFormat, Ss58AddressFormatRegistry, TokenRegistry};

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

#[cfg(feature = "std")]
#[test]
fn enum_to_name_and_back() {
	use std::convert::TryInto;
	for name in Ss58AddressFormat::all_names() {
		let val: Ss58AddressFormatRegistry = (*name).try_into().expect(name);
		assert_eq!(name, &val.to_string());
	}
}

#[test]
fn prefix() {
	let dot: Ss58AddressFormat = Ss58AddressFormatRegistry::PolkadotAccount.into();
	assert_eq!(dot.prefix(), 0);
	let ksm: Ss58AddressFormat = Ss58AddressFormatRegistry::KusamaAccount.into();
	assert_eq!(ksm.prefix(), 2);
}

#[test]
fn tokens() {
	let polka = Ss58AddressFormatRegistry::PolkadotAccount;
	assert_eq!(polka.tokens(), vec![TokenRegistry::Dot]);
	let kusama = Ss58AddressFormatRegistry::KusamaAccount;
	assert_eq!(kusama.tokens(), vec![TokenRegistry::Ksm]);
	let darwinia = Ss58AddressFormatRegistry::DarwiniaAccount;
	assert_eq!(darwinia.tokens(), vec![TokenRegistry::Ring, TokenRegistry::Kton]);
	let n46 = Ss58AddressFormatRegistry::Reserved46Account;
	assert_eq!(n46.tokens(), vec![]);
}
