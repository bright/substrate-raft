use super::config;

pub struct AuthorityData {
	pub current_slot: usize,
	pub config: config::Config,
}

impl AuthorityData {
	pub fn new() -> AuthorityData {
		AuthorityData { current_slot: 0, config: config::Config::new() }
	}

	pub fn create(cfg: config::Config) -> AuthorityData {
		AuthorityData { current_slot: 0, config: cfg }
	}
}
