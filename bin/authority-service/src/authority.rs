use super::config;

pub struct AuthorityData {
	pub current_slot: i64,
    pub current_round: i64,
    pub current_session: i64,
	pub config: config::Config,
}

impl AuthorityData {
	pub fn new() -> AuthorityData {
		AuthorityData { current_slot: -1, current_round: -1, current_session: -1, config: config::Config::new() }
	}

	pub fn create(cfg: config::Config) -> AuthorityData {
		AuthorityData { current_slot: -1, current_round: -1, current_session: -1, config: cfg }
	}
}
