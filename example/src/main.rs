use sp_debug_derive::RuntimeDebug;
use ss58_registry::ss58_registry;

ss58_registry!();

pub fn main() {
    assert!(Ss58AddressFormat::Custom(1) != Ss58AddressFormat::Custom(2));
}
