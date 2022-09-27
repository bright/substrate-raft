use async_trait::async_trait;
use log::{debug, error};
use reqwest::{Client, ClientBuilder};
use sp_authority_permission::{AuthorityPermissionCmd, PermissionType};
use sp_consensus_slots::Slot;
use std::sync::mpsc::Receiver;

pub struct RemoteAuthorityPermissionResolver {
	client: Client,
	base_url: String,
}

impl RemoteAuthorityPermissionResolver {
	pub fn new(base_url: &str) -> RemoteAuthorityPermissionResolver {
		let client = ClientBuilder::new().build().expect("Could not create client");
		RemoteAuthorityPermissionResolver { client, base_url: base_url.to_owned() }
	}

	async fn resolve_slot(&self, slot: Slot) -> bool {
		match self.do_resolve_slot(slot).await {
			Ok(result) => result,
			Err(e) => {
				error!(
					target: "permission-resolver",
					"Could not resolve permission, reason: {}", e);
				false
			},
		}
	}

	async fn resolve_round(&self, round: u64) -> bool {
		match self.do_resolve_round(round).await {
			Ok(result) => result,
			Err(e) => {
				error!(
					target: "permission-resolver",
					"Could not resolve permission, reason: {}", e);
				false
			},
		}
	}

	async fn do_resolve_slot(&self, slot: Slot) -> Result<bool, String> {
		debug!(target: "permission-resolver", "Checking slot {} permission...", slot);
		let url = format!("{}/authorize/slot/{}", self.base_url, slot);
		let resp = self
			.client
			.put(url)
			.send()
			.await
			.map_err(|_| "Could not reach out to remote service")?;
		let can: bool =
			resp.text().await.expect("").parse().map_err(|_| "Could not parse response")?;
		debug!(target: "permission-resolver", "Got slot {} permission: {}", slot, can);
		Ok(can)
	}

	async fn do_resolve_round(&self, round: u64) -> Result<bool, String> {
		debug!(target: "permission-resolver", "Checking round  {} permission...", round);
		let url = format!("{}/authorize/round/{}", self.base_url, round);
		let resp = self
			.client
			.put(url)
			.send()
			.await
			.map_err(|_| "Could not reach out to remote service")?;
		let can: bool =
			resp.text().await.expect("").parse().map_err(|_| "Could not parse response")?;
		debug!(target: "permission-resolver", "Got round {} permission: {}", round, can);
		Ok(can)
	}
}

pub async fn permission_resolver_future(
	remote_authority: String,
	receiver: Receiver<AuthorityPermissionCmd>,
) {
	let client = RemoteAuthorityPermissionResolver::new(&remote_authority);
	loop {
		let cmd = receiver.recv().expect("Could not receive command");
		let can = match cmd.permission_type {
			PermissionType::ROUND(round) => client.resolve_round(round).await,
			PermissionType::SLOT(slot) => client.resolve_slot(slot).await,
		};
		cmd.respond_to.send(can).expect("Could not send to channel");
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use httpmock::MockServer;

	#[tokio::test]
	async fn test_remote_permits_slot() {
		let server = MockServer::start();
		let mock = server.mock(|when, then| {
			when.path("/authorize/slot/1");
			then.status(200).body("true");
		});
		let permission_resolver = RemoteAuthorityPermissionResolver::new(&server.base_url());
		let permission = permission_resolver.resolve_slot(1.into()).await;

		mock.assert();
		assert!(permission)
	}

	#[tokio::test]
	async fn test_remote_denies_slot() {
		let server = MockServer::start();
		let mock = server.mock(|when, then| {
			when.path("/authorize/slot/1");
			then.status(200).body("false");
		});
		let permission_resolver = RemoteAuthorityPermissionResolver::new(&server.base_url());
		let permission = permission_resolver.resolve_slot(1.into()).await;

		mock.assert();
		assert!(!permission)
	}

	#[tokio::test]
	async fn test_remote_permits_round() {
		let server = MockServer::start();
		let mock = server.mock(|when, then| {
			when.path("/authorize/round/1");
			then.status(200).body("true");
		});
		let permission_resolver = RemoteAuthorityPermissionResolver::new(&server.base_url());
		let permission = permission_resolver.resolve_round(1).await;

		mock.assert();
		assert!(permission)
	}

	#[tokio::test]
	async fn test_remote_denies_round() {
		let server = MockServer::start();
		let mock = server.mock(|when, then| {
			when.path("/authorize/round/1");
			then.status(200).body("false");
		});
		let permission_resolver = RemoteAuthorityPermissionResolver::new(&server.base_url());
		let permission = permission_resolver.resolve_round(1).await;

		mock.assert();
		assert!(!permission)
	}

	#[tokio::test]
	async fn test_permission_denied_in_case_of_integration_error() {
		let permission_resolver = RemoteAuthorityPermissionResolver::new("localhost");
		let permission = permission_resolver.resolve_slot(1.into()).await;
		assert!(!permission)
	}
}
