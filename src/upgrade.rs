pub use std::{cell::RefCell, mem};

pub(crate) use ic_cdk::storage;
pub(crate) use ic_cdk_macros::{pre_upgrade, post_upgrade};

thread_local! {
    pub static STATE: std::cell::RefCell<crate::model::Data>  = RefCell::default();
}

#[pre_upgrade]
fn pre_upgrade() {
    let state = STATE.with(|state| mem::take(&mut *state.borrow_mut()));
    storage::stable_save(state).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let state = storage::stable_restore().unwrap();
    STATE.with(|state0| *state0.borrow_mut() = state);
}

