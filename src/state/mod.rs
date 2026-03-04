mod persistence;
mod store;

pub use store::{AppState, Toast, ToastVariant, provide_app_state, use_app_state};
