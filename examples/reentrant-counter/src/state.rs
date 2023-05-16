// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use linera_sdk::{application_state, views::RegisterView};

/// The application state.
#[application_state(view)]
pub struct ReentrantCounter {
    pub value: RegisterView<u128>,
}
