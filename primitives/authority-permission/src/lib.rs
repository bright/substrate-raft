// This file is part of Substrate.

// Copyright (C) 2020-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use sp_consensus_slots::Slot;
use std::sync::{mpsc, Mutex};

pub struct AuthorityPermissionHandle {
	pub requests: Mutex<mpsc::Sender<AuthorityPermissionCmd>>,
}

type AuthorityCmdWithReceiver = (AuthorityPermissionCmd, mpsc::Receiver<bool>);

impl AuthorityPermissionHandle {
	pub fn has(&self, cmd: AuthorityCmdWithReceiver) -> bool {
		self.requests
			.lock()
			.expect("Could not lock")
			.send(cmd.0)
			.expect("Could not send command");
		cmd.1.recv().expect("Could not receive result")
	}
}

pub enum PermissionType {
	SLOT(Slot),
	ROUND(u64),
}

pub struct AuthorityPermissionCmd {
	pub permission_type: PermissionType,
	pub respond_to: mpsc::Sender<bool>,
}

impl AuthorityPermissionCmd {
	pub fn prepare(resolve_type: PermissionType) -> AuthorityCmdWithReceiver {
		let (sender, receiver) = mpsc::channel();
		(AuthorityPermissionCmd { permission_type: resolve_type, respond_to: sender }, receiver)
	}
}
