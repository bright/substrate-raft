use async_trait::async_trait;
use sp_authority_permission::PermissionResolver;
use sp_consensus_slots::Slot;
use std::ops::Deref;
use log::{error};
use reqwest::{Client, ClientBuilder};

pub struct RemoteAuthorityPermissionResolver {
	client: Client,
	base_url: String
}

impl RemoteAuthorityPermissionResolver {

	pub fn new(base_url: &str) -> RemoteAuthorityPermissionResolver {
		let client = ClientBuilder::new().build().expect("Could not create client");
		RemoteAuthorityPermissionResolver {
			client,
			base_url:  base_url.to_owned()
		}
	}

	async fn do_resolve(&self, slot: Slot) -> Result<bool, String> {
		let url = format!("{}/authorize/{}", self.base_url,  slot);
		let resp = self.client.put(url).send().await.map_err(|_| "Could not reach out to remote service")?;
		let can: bool = resp.text().await.expect("").parse().map_err(|_| "Could not parse response")?;
		Ok(can)
	}
}

#[async_trait]
impl PermissionResolver for RemoteAuthorityPermissionResolver {
	async fn resolve(&self, slot: Slot) -> bool {
		match self.do_resolve(slot).await {
			Ok(result) => result,
			Err(e) => {
				error!(
					target: "permission-resolver",
					"Could not resolve permission, reason: {}", e);
				false
			}
		}
	}
}

pub struct OddSlotPermissionResolver {}

#[async_trait]
impl PermissionResolver for OddSlotPermissionResolver {
	async fn resolve(&self, slot: Slot) -> bool {
		slot.deref() % 2 == 0
	}
}

#[cfg(test)]
mod tests {
	use httpmock::MockServer;
	use super::*;
	use sp_authority_permission::PermissionResolver;

	#[tokio::test]
	async fn test_even_slots() {
		let permission_resolver = OddSlotPermissionResolver {};
		let slot_no = 1.into();
		assert!(!permission_resolver.resolve(slot_no).await)
	}

	#[tokio::test]
	async fn test_odd_slots() {
		let permission_resolver = OddSlotPermissionResolver {};
		let slot_no = 2.into();
		assert!(permission_resolver.resolve(slot_no).await)
	}

	#[tokio::test]
	async fn test_remote_permits() {
		let server = MockServer::start();
		let mock = server.mock(|when, then|{
			when.path("/authorize/1");
			then.status(200).body("true");
		});
		let permission_resolver = RemoteAuthorityPermissionResolver::new(&server.base_url());
		let permission = permission_resolver.resolve(1.into()).await;

		mock.assert();
		assert!(permission)
	}


	#[tokio::test]
	async fn test_remote_denies() {
		let server = MockServer::start();
		let mock = server.mock(|when, then|{
			when.path("/authorize/1");
			then.status(200).body("false");
		});
		let permission_resolver = RemoteAuthorityPermissionResolver::new(&server.base_url());
		let permission = permission_resolver.resolve(1.into()).await;

		mock.assert();
		assert!(!permission)
	}

	#[tokio::test]
	async fn test_permission_denied_in_case_of_integration_error() {
		let permission_resolver = RemoteAuthorityPermissionResolver::new("localhost");
		let permission = permission_resolver.resolve(1.into()).await;
		assert!(!permission)
	}

}
