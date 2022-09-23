#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use std::{
	env,
	sync::{Arc, Mutex},
};

mod authority;
mod config;
mod handler;

#[launch]
fn rocket() -> _ {
	let args: Vec<String> = env::args().collect();
	let mut data = authority::AuthorityData::new();
	if let Some(path) = args.last() {
		if let Some(cfg) = config::Config::from_json_file(path) {
			data = authority::AuthorityData::create(cfg);
		}
	}

	rocket::build()
		.manage(Arc::new(Mutex::new(data)))
        .mount("/", routes![handler::authorize_slot, handler::authorize_round, handler::authorize_session, handler::authorize_fix_order])
}
