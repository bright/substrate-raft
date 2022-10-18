use async_trait::async_trait;
use log::{debug, error};
use sp_authority_permission::PermissionResolver;
use sp_consensus_slots::Slot;
use tikv_client::{transaction::Client, Error, TransactionClient, Value};

pub mod cache;

pub use cache::PermissionResolverCache;

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

pub struct RemoteAuthorityPermissionResolver {
	client: Client,
}

impl RemoteAuthorityPermissionResolver {
	pub async fn new(pd_addresses: Vec<String>) -> RemoteAuthorityPermissionResolver {
		let client = TransactionClient::new(pd_addresses).await.expect("Could not create client");
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
