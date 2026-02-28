mod persistence;
mod store;

pub use persistence::{load_note_sets, load_settings, save_note_sets, save_settings};
pub use store::{AppState, Settings, Toast, ToastVariant, provide_app_state, use_app_state};
