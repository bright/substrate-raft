// MIT License

// Copyright (c) 2023 Bright Inventions

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
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
