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
use async_trait::async_trait;
use sp_consensus_slots::Slot;

#[async_trait]
pub trait PermissionResolver: Send + Sync {
	async fn resolve_slot(&self, slot: Slot) -> bool;
	async fn resolve_round(&self, round: u64) -> bool;
	async fn resolve_session(&self, session_index: u32) -> bool;
}

impl std::fmt::Debug for dyn PermissionResolverFactory {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "PermissionResolverFactory")
	}
}

#[async_trait]
pub trait PermissionResolverFactory {
	async fn create(&self) -> Box<dyn PermissionResolver>;
}

pub struct AlwaysPermissionGranted {}

#[async_trait]
impl PermissionResolver for AlwaysPermissionGranted {
	async fn resolve_slot(&self, _: Slot) -> bool {
		true
	}

	async fn resolve_round(&self, _: u64) -> bool {
		true
	}

	async fn resolve_session(&self, _: u32) -> bool {
		true
	}
}

pub struct AlwaysPermissionGrantedFactory {}

#[async_trait]
impl PermissionResolverFactory for AlwaysPermissionGrantedFactory {
	async fn create(&self) -> Box<dyn PermissionResolver> {
		Box::new(AlwaysPermissionGranted {})
	}
}

pub struct NeverPermissionGranted {}

#[async_trait]
impl PermissionResolver for NeverPermissionGranted {
	async fn resolve_slot(&self, _: Slot) -> bool {
		false
	}

	async fn resolve_round(&self, _: u64) -> bool {
		false
	}

	async fn resolve_session(&self, _: u32) -> bool {
		false
	}
}

pub struct NeverPermissionGrantedFactory {}

#[async_trait]
impl PermissionResolverFactory for NeverPermissionGrantedFactory {
	async fn create(&self) -> Box<dyn PermissionResolver> {
		Box::new(NeverPermissionGranted {})
	}
}
