// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::ReentrantCounter;
use async_trait::async_trait;
use linera_sdk::{QueryContext, Service, ViewStateStorage};
use std::sync::Arc;
use thiserror::Error;

linera_sdk::service!(ReentrantCounter);

#[async_trait]
impl Service for ReentrantCounter {
    type Error = Error;
    type Storage = ViewStateStorage<Self>;

    async fn query_application(
        self: Arc<Self>,
        _context: &QueryContext,
        argument: &[u8],
    ) -> Result<Vec<u8>, Self::Error> {
        let value = self.value.get();
        match argument {
            &[] => Ok(bcs::to_bytes(&value).expect("Serialization should not fail")),
            _ => Err(Error::InvalidQuery),
        }
    }
}

/// An error that can occur during the contract execution.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum Error {
    /// Invalid query argument; Counter application only supports a single (empty) query.
    #[error("Invalid query argument; Counter application only supports a single (empty) query")]
    InvalidQuery,
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::Error;
    use crate::ReentrantCounter;
    use futures::FutureExt;
    use linera_sdk::{
        base::ChainId,
        test,
        views::{View, ViewStorageContext},
        QueryContext, Service,
    };
    use std::sync::Arc;
    use webassembly_test::webassembly_test;

    #[webassembly_test]
    fn query() {
        test::mock_key_value_store();
        let value = 61_098_721_u128;
        let mut counter = ReentrantCounter::load(ViewStorageContext::default())
            .now_or_never()
            .unwrap()
            .expect("Failed to load Counter");
        counter.value.set(value);
        let counter = Arc::new(counter);
        let result = counter
            .query_application(&dummy_query_context(), &[])
            .now_or_never()
            .expect("Query should not await anything");

        let expected_response =
            bcs::to_bytes(&value).expect("Counter value could not be serialized");

        assert_eq!(result, Ok(expected_response));
    }

    #[webassembly_test]
    fn invalid_query() {
        test::mock_key_value_store();
        let value = 4_u128;
        let mut counter = ReentrantCounter::load(ViewStorageContext::default())
            .now_or_never()
            .unwrap()
            .expect("Failed to load Counter");
        counter.value.set(value);
        let counter = Arc::new(counter);

        let dummy_argument = [2];
        let result = counter
            .query_application(&dummy_query_context(), &dummy_argument)
            .now_or_never()
            .expect("Query should not await anything");

        assert_eq!(result, Err(Error::InvalidQuery));
    }

    fn dummy_query_context() -> QueryContext {
        QueryContext {
            chain_id: ChainId([0; 4].into()),
        }
    }
}
