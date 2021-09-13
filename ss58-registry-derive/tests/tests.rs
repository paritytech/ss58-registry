use sp_debug_derive::RuntimeDebug;
use ss58_registry_derive::ss58_registry_derive;

#[test]
fn test_can_construct() {
    ss58_registry_derive!();
}
