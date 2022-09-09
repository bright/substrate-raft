use async_trait::async_trait;
use sp_authority_permission::PermissionResolver;
use sp_consensus_slots::Slot;
use std::ops::Deref;

pub struct BasicPermissionResolver {}

#[async_trait]
impl PermissionResolver for BasicPermissionResolver {
	async fn resolve(&self, slot: Slot) -> bool {
		slot.deref() % 2 == 0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_authority_permission::PermissionResolver;

	#[tokio::test]
	async fn test_even_slots() {
		let permission_resolver = BasicPermissionResolver {};
		let slot_no = 1.into();
		assert!(!permission_resolver.resolve(slot_no).await)
	}

	#[tokio::test]
	async fn test_odd_slots() {
		let permission_resolver = BasicPermissionResolver {};
		let slot_no = 2.into();
		assert!(permission_resolver.resolve(slot_no).await)
	}
}
