use async_trait::async_trait;
use log::{debug, error};
use sp_authority_permission::PermissionResolver;
use sp_consensus_slots::Slot;
use tikv_client::{transaction::Client, Error, Timestamp, Transaction, TransactionClient, Value};

pub mod cache;
pub mod metrics;

pub use cache::PermissionResolverCache;
pub use metrics::PermissionResolverMetrics;

enum Key {
	SLOT,
	SESSION,
	ROUND,
}

impl Key {
	fn as_str(&self) -> &'static str {
		match self {
			Key::SLOT => "slot",
			Key::SESSION => "session",
			Key::ROUND => "round",
		}
	}
}

#[async_trait]
pub trait TiKVClient: Send + Sync {
	async fn begin_optimistic(&self) -> Result<Box<dyn TiKVTransaction>, Error>;
}

#[async_trait]
pub trait TiKVTransaction: Send {
	async fn get_for_update(&mut self, key: String) -> Result<Option<Value>, Error>;
	async fn put(&mut self, key: String, value: Vec<u8>) -> Result<(), Error>;
	async fn commit(&mut self) -> Result<Option<Timestamp>, Error>;
	async fn rollback(&mut self) -> Result<(), Error>;
}

pub async fn create_remote_authority_provider(
	pd_addresses: Vec<String>,
) -> RemoteAuthorityPermissionResolver {
	let client = TransactionClient::new(pd_addresses).await.expect("Could not create client");
	RemoteAuthorityPermissionResolver { client: Box::new(TiKVClientProxy { inner: client }) }
}

struct TiKVClientProxy {
	inner: Client,
}

#[async_trait]
impl TiKVClient for TiKVClientProxy {
	async fn begin_optimistic(&self) -> Result<Box<dyn TiKVTransaction>, Error> {
		Ok(Box::new(TiKVTransactionProxy { inner: self.inner.begin_optimistic().await? }))
	}
}

struct TiKVTransactionProxy {
	inner: Transaction,
}

#[async_trait]
impl TiKVTransaction for TiKVTransactionProxy {
	async fn get_for_update(&mut self, key: String) -> Result<Option<Value>, Error> {
		self.inner.get_for_update(key).await
	}

	async fn put(&mut self, key: String, value: Vec<u8>) -> Result<(), Error> {
		self.inner.put(key, value).await
	}

	async fn commit(&mut self) -> Result<Option<Timestamp>, Error> {
		self.inner.commit().await
	}

	async fn rollback(&mut self) -> Result<(), Error> {
		self.inner.rollback().await
	}
}

pub struct RemoteAuthorityPermissionResolver {
	client: Box<dyn TiKVClient>,
}

impl RemoteAuthorityPermissionResolver {
	pub async fn new(client: Box<dyn TiKVClient>) -> RemoteAuthorityPermissionResolver {
		RemoteAuthorityPermissionResolver { client }
	}

	///Tries to optimistically update the value if it's less than current,
	/// if the operation is successful we treat it as permission granted.
	async fn do_resolve(&self, key: Key, value: u64) -> Result<bool, String> {
		debug!(target: "permission-resolver", "Checking {} {} permission...", key.as_str(), value);
		let mut txn = self
			.client
			.begin_optimistic()
			.await
			.map_err(|e| format!("Could not start transaction, reason: {}", e))?;
		let can = txn
			.get_for_update(key.as_str().to_owned())
			.await
			.map_err(|e| format!("Could not get {} value for update, reason: {}", key.as_str(), e))?
			.map_or(true, |v| value > deserialize_u64(v));
		if can {
			txn.put(key.as_str().to_owned(), u64::to_be_bytes(value).to_vec())
				.await
				.map_err(|e| format!("Could not put {} value, reason {}", key.as_str(), e))?;
			match txn.commit().await {
				Ok(_) => {},
				Err(ref e) => {
					match e {
						Error::KeyError(inner_e) => {
							if inner_e.conflict.is_some() {
								//conflict indicates that somebody was faster reserving
								// slot/session/round
								return Ok(false)
							} else {
								return Err(format!("Could not commit transaction, reason {}", e))
							}
						},
						e => return Err(format!("Could not commit transaction, reason {}", e)),
					}
				},
			}
		} else {
			txn.rollback()
				.await
				.map_err(|e| format!("Could not rollback transaction, reason {}", e))?;
		}
		Ok(can)
	}
}

fn deserialize_u64(value: Value) -> u64 {
	let mut buf = [0u8; 8];
	let len = 8.min(value.len());
	buf[..len].copy_from_slice(&value[..len]);
	u64::from_be_bytes(buf)
}

#[async_trait]
impl PermissionResolver for RemoteAuthorityPermissionResolver {
	async fn resolve_slot(&self, slot: Slot) -> bool {
		match self.do_resolve(Key::SLOT, slot.into()).await {
			Ok(result) => result,
			Err(e) => {
				error!(
               target: "permission-resolver",
               "Could not resolve slot permission, reason: {}", e);
				false
			},
		}
	}

	async fn resolve_round(&self, round: u64) -> bool {
		match self.do_resolve(Key::ROUND, round).await {
			Ok(result) => result,
			Err(e) => {
				error!(
               target: "permission-resolver",
               "Could not resolve round permission, reason: {}", e);
				false
			},
		}
	}

	async fn resolve_session(&self, session_index: u32) -> bool {
		match self.do_resolve(Key::SESSION, session_index.into()).await {
			Ok(result) => result,
			Err(e) => {
				error!(
               target: "permission-resolver",
               "Could not resolve session permission, reason: {}", e);
				false
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_authority_permission::PermissionResolver;

	struct MockedTiKVClient {
		slot: Option<Slot>,
		round: Option<u64>,
		session: Option<u32>,
	}

	#[async_trait]
	impl TiKVClient for MockedTiKVClient {
		async fn begin_optimistic(&self) -> Result<Box<dyn TiKVTransaction>, Error> {
			Ok(Box::new(MockedTiKVTransaction {
				slot: *&self.slot,
				round: *&self.round,
				session: *&self.session,
			}))
		}
	}

	struct MockedTiKVTransaction {
		slot: Option<Slot>,
		round: Option<u64>,
		session: Option<u32>,
	}

	#[async_trait]
	impl TiKVTransaction for MockedTiKVTransaction {
		async fn get_for_update(&mut self, key: String) -> Result<Option<Value>, Error> {
			if key == Key::SLOT.as_str() {
				Ok(self.slot.map(|s| u64::to_be_bytes(s.into()).to_vec()))
			} else if key == Key::ROUND.as_str() {
				Ok(self.round.map(|r| u64::to_be_bytes(r).to_vec()))
			} else if key == Key::SESSION.as_str() {
				Ok(self.session.map(|s| u64::to_be_bytes(s.into()).to_vec()))
			} else {
				Ok(None)
			}
		}

		async fn put(&mut self, _: String, _: Vec<u8>) -> Result<(), Error> {
			Ok(())
		}

		async fn commit(&mut self) -> Result<Option<Timestamp>, Error> {
			Ok(Some(Timestamp::default()))
		}

		async fn rollback(&mut self) -> Result<(), Error> {
			Ok(())
		}
	}

	#[tokio::test]
	async fn test_permits_round_if_higher() {
		let client = MockedTiKVClient { slot: None, round: Some(1), session: None };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(resolver.resolve_round(2).await)
	}

	#[tokio::test]
	async fn test_denies_round_if_equal() {
		let client = MockedTiKVClient { slot: None, round: Some(1), session: None };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(!resolver.resolve_round(1).await)
	}

	#[tokio::test]
	async fn test_denies_round_if_lower() {
		let client = MockedTiKVClient { slot: None, round: Some(1), session: None };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(!resolver.resolve_round(0).await)
	}

	#[tokio::test]
	async fn test_permits_session_if_higher() {
		let client = MockedTiKVClient { slot: None, round: None, session: Some(1) };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(resolver.resolve_session(2).await)
	}

	#[tokio::test]
	async fn test_denies_session_if_equal() {
		let client = MockedTiKVClient { slot: None, round: None, session: Some(1) };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(!resolver.resolve_session(1).await)
	}

	#[tokio::test]
	async fn test_denies_session_if_lower() {
		let client = MockedTiKVClient { slot: None, round: None, session: Some(1) };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(!resolver.resolve_session(0).await)
	}

	#[tokio::test]
	async fn test_permits_slot_if_higher() {
		let client = MockedTiKVClient { slot: Some(1.into()), round: None, session: None };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(resolver.resolve_slot(2.into()).await)
	}

	#[tokio::test]
	async fn test_denies_slot_if_equal() {
		let client = MockedTiKVClient { slot: Some(1.into()), round: None, session: None };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(!resolver.resolve_slot(1.into()).await)
	}

	#[tokio::test]
	async fn test_denies_slot_if_lower() {
		let client = MockedTiKVClient { slot: Some(1.into()), round: None, session: None };
		let resolver = RemoteAuthorityPermissionResolver::new(Box::new(client)).await;
		assert!(!resolver.resolve_slot(0.into()).await)
	}
}
