use async_trait::async_trait;
use sp_authority_permission::PermissionResolver;
use sp_consensus_slots::Slot;
use std::sync::Mutex;

type Cache = Mutex<Option<(u64, bool)>>;

/// Cache for permission resolver. It's holds vales for the last requests of the slot/round/session.
/// This prevent from frequently requesting of the permission resolver.
pub struct PermissionResolverCache {
	resolver: Box<dyn PermissionResolver>,
	slot: Cache,
	round: Cache,
	session: Cache,
}

impl PermissionResolverCache {
	/// Create a new instance of the cache.
	pub fn new(resolver: Box<dyn PermissionResolver>) -> PermissionResolverCache {
		PermissionResolverCache {
			resolver,
			slot: Mutex::new(None),
			round: Mutex::new(None),
			session: Mutex::new(None),
		}
	}

	/// Check the permission of the slot/round/session.
	fn check_permission(cache: &Cache, value: u64) -> Option<bool> {
		if let Some(v) = *cache.lock().unwrap() {
			if v.0 == value {
				return Some(v.1.clone())
			}
		}
		return None
	}

	/// Set the cached permission of the slot/round/session.
	fn set_permission(cache: &Cache, value: u64, permission: bool) {
		*cache.lock().unwrap() = Some((value, permission));
	}
}

#[async_trait]
impl PermissionResolver for PermissionResolverCache {
	async fn resolve_slot(&self, slot: Slot) -> bool {
		if let Some(permission) = PermissionResolverCache::check_permission(&self.slot, slot.into())
		{
			return permission
		}

		let permission = self.resolver.resolve_slot(slot).await;
		PermissionResolverCache::set_permission(&self.slot, slot.into(), permission.clone());
		return permission
	}

	async fn resolve_round(&self, round: u64) -> bool {
		if let Some(permission) =
			PermissionResolverCache::check_permission(&self.round, round.into())
		{
			return permission
		}

		let permission = self.resolver.resolve_round(round).await;
		PermissionResolverCache::set_permission(&self.round, round.clone(), permission.clone());
		permission
	}

	async fn resolve_session(&self, session_index: u32) -> bool {
		if let Some(permission) =
			PermissionResolverCache::check_permission(&self.session, session_index.into())
		{
			return permission
		}

		let permission = self.resolver.resolve_session(session_index).await;
		PermissionResolverCache::set_permission(
			&self.session,
			session_index.into(),
			permission.clone(),
		);
		permission
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_authority_permission::PermissionResolver;
	use std::sync::Arc;

	struct PermissionCounters {
		slot: u64,
		round: u64,
		session: u64,
	}

	type PermissionCountersArc = Arc<Mutex<PermissionCounters>>;

	struct PermissionResolverMock {
		call_counter: PermissionCountersArc,
	}

	impl PermissionResolverMock {
		fn new(counter: PermissionCountersArc) -> PermissionResolverMock {
			PermissionResolverMock { call_counter: counter }
		}
	}

	#[async_trait]
	impl PermissionResolver for PermissionResolverMock {
		async fn resolve_slot(&self, _: Slot) -> bool {
			self.call_counter.lock().unwrap().slot += 1;
			true
		}

		async fn resolve_round(&self, _: u64) -> bool {
			self.call_counter.lock().unwrap().round += 1;
			true
		}

		async fn resolve_session(&self, _: u32) -> bool {
			self.call_counter.lock().unwrap().session += 1;
			true
		}
	}

	#[tokio::test]
	async fn test_permission_resolver_cache_slot() {
		let counters = Arc::new(Mutex::new(PermissionCounters { slot: 0, round: 0, session: 0 }));
		let resolver =
			PermissionResolverCache::new(Box::new(PermissionResolverMock::new(counters.clone())));

		assert_eq!(counters.lock().unwrap().slot, 0);

		// Normal request, should be forwarded to mock.
		let permission = resolver.resolve_slot(0.into()).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().slot, 1);

		// Re-request for the same value, should call cache instead of mock.
		let permission = resolver.resolve_slot(0.into()).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().slot, 1);

		// Request for new slot, should be forwarded to mock.
		let permission = resolver.resolve_slot(1.into()).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().slot, 2);
	}

	#[tokio::test]
	async fn test_permission_resolver_cache_round() {
		let counters = Arc::new(Mutex::new(PermissionCounters { slot: 0, round: 0, session: 0 }));
		let resolver =
			PermissionResolverCache::new(Box::new(PermissionResolverMock::new(counters.clone())));

		// Test initial values
		assert_eq!(counters.lock().unwrap().round, 0);

		// Normal request, should be forwarded to mock.
		let permission = resolver.resolve_round(0).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().round, 1);

		// Re-request for the same value, should call cache instead of mock.
		let permission = resolver.resolve_round(0).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().round, 1);

		// Request for new round, should be forwarded to mock.
		let permission = resolver.resolve_round(1).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().round, 2);
	}

	#[tokio::test]
	async fn test_permission_resolver_cache_session() {
		let counters = Arc::new(Mutex::new(PermissionCounters { slot: 0, round: 0, session: 0 }));
		let resolver =
			PermissionResolverCache::new(Box::new(PermissionResolverMock::new(counters.clone())));

		// Test initial values
		assert_eq!(counters.lock().unwrap().session, 0);

		// Normal request, should be forwarded to mock.
		let permission = resolver.resolve_session(0).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().session, 1);

		// Re-request for the same value, should call cache instead of mock.
		let permission = resolver.resolve_session(0).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().session, 1);

		// Request for new session, should be forwarded to mock.
		let permission = resolver.resolve_session(1).await;
		assert!(permission);
		assert_eq!(counters.lock().unwrap().session, 2);
	}
}
