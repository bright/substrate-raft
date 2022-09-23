use rocket::{self, http::Status, State};

use std::sync::{Arc, Mutex};

use super::authority;

#[put("/authorize_fix_order/<name>")]
pub fn authorize_fix_order(
	name: &str,
	data: &State<Arc<Mutex<authority::AuthorityData>>>,
) -> Result<String, Status> {
	let mut data = data.lock().unwrap();
	if data.config.is_authorized(&name) {
		data.config.next();
		return Result::Ok(true.to_string())
	}
	Result::Ok(false.to_string())
}

#[put("/authorize/slot/<slot_nr>")]
pub fn authorize_slot(
	slot_nr: i64,
	data: &State<Arc<Mutex<authority::AuthorityData>>>,
) -> Result<String, Status> {
	let mut data = data.lock().unwrap();
	if data.current_slot < slot_nr {
		data.current_slot = slot_nr;
		return Result::Ok(true.to_string())
	}
	Result::Ok(false.to_string())
}
#[put("/authorize/round/<round_nr>")]
pub fn authorize_round(
	round_nr: i64,
	data: &State<Arc<Mutex<authority::AuthorityData>>>,
) -> Result<String, Status> {
	let mut data = data.lock().unwrap();
	if data.current_round < round_nr {
		data.current_round = round_nr;
		return Result::Ok(true.to_string())
	}
	Result::Ok(false.to_string())
}

#[put("/authorize/session/<session_nr>")]
pub fn authorize_session(
	session_nr: i64,
	data: &State<Arc<Mutex<authority::AuthorityData>>>,
) -> Result<String, Status> {
	let mut data = data.lock().unwrap();
	if data.current_session < 0 || data.current_session < session_nr {
		data.current_session = session_nr;
		return Result::Ok(true.to_string())
	}
	Result::Ok(false.to_string())
}

#[cfg(test)]
mod test {
	use super::{rocket, *};
	use crate::config;
	use rocket::http::Status;

	#[test]
	fn test_authorize() {
		use rocket::local::blocking::Client;

		let rocket = rocket::build()
			.manage(Arc::new(Mutex::new(authority::AuthorityData::new())))
			.mount("/", routes![authorize_slot]);

		let client = Client::tracked(rocket).unwrap();
		let response = client.put("/authorize/slot/1").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("true".into()));

		let response = client.put("/authorize/slot/1").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("false".into()));

		let response = client.put("/authorize/slot/2").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("true".into()));
	}

    #[test]
	fn test_authorize_session() {
		use rocket::local::blocking::Client;

		let rocket = rocket::build()
			.manage(Arc::new(Mutex::new(authority::AuthorityData::new())))
			.mount("/", routes![authorize]);

		let client = Client::tracked(rocket).unwrap();
		let response = client.put("/authorize/session/1").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("true".into()));

		let response = client.put("/authorize/session/1").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("false".into()));

		let response = client.put("/authorize/session/2").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("true".into()));
	}

	#[test]
	fn test_authorize_fix_order() {
		use rocket::local::blocking::Client;

		let data = r#"
        {
            "nodes": [
                "node1",
                "node2",
                "node3"
            ],
            "order": [
                "node3",
                "node1",
                "node2"
            ]
        }"#;

		let cfg = config::Config::from_json(data).unwrap();
		let rocket = rocket::build()
			.manage(Arc::new(Mutex::new(authority::AuthorityData::create(cfg))))
			.mount("/", routes![authorize_fix_order]);

		let client = Client::tracked(rocket).unwrap();
		let response = client.put("/authorize_fix_order/node1").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("false".into()));

		let response = client.put("/authorize_fix_order/node3").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("true".into()));

		let response = client.put("/authorize_fix_order/node2").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("false".into()));

		let response = client.put("/authorize_fix_order/node1").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("true".into()));

		let response = client.put("/authorize_fix_order/node2").dispatch();
		assert_eq!(response.status(), Status::Ok);
		assert_eq!(response.into_string(), Some("true".into()));
	}
}
