use web_sys::window;

use crate::models::NoteSet;
use super::store::Settings;

const STORAGE_KEY_SETS: &str = "ecash-tracker-sets";
const STORAGE_KEY_SETTINGS: &str = "ecash-tracker-settings";

fn get_local_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

pub fn load_note_sets() -> Result<Vec<NoteSet>, String> {
    let storage = get_local_storage().ok_or("LocalStorage not available")?;

    match storage.get_item(STORAGE_KEY_SETS) {
        Ok(Some(data)) => {
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse note sets: {}", e))
        }
        Ok(None) => Ok(Vec::new()),
        Err(e) => Err(format!("Failed to read from localStorage: {:?}", e)),
    }
}

pub fn save_note_sets(sets: &[NoteSet]) -> Result<(), String> {
    let storage = get_local_storage().ok_or("LocalStorage not available")?;

    let data = serde_json::to_string(sets)
        .map_err(|e| format!("Failed to serialize note sets: {}", e))?;

    storage
        .set_item(STORAGE_KEY_SETS, &data)
        .map_err(|e| format!("Failed to write to localStorage: {:?}", e))
}

pub fn load_settings() -> Result<Settings, String> {
    let storage = get_local_storage().ok_or("LocalStorage not available")?;

    match storage.get_item(STORAGE_KEY_SETTINGS) {
        Ok(Some(data)) => {
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse settings: {}", e))
        }
        Ok(None) => Ok(Settings::default()),
        Err(e) => Err(format!("Failed to read from localStorage: {:?}", e)),
    }
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let storage = get_local_storage().ok_or("LocalStorage not available")?;

    let data = serde_json::to_string(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    storage
        .set_item(STORAGE_KEY_SETTINGS, &data)
        .map_err(|e| format!("Failed to write to localStorage: {:?}", e))
}
