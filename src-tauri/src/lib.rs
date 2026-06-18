pub mod shared;

pub fn run() {
    tauri::Builder::default()
        // SCRIBE-3: register secrets commands (keychain get/set/has/delete for API keys)
        // SCRIBE-4: register platform commands (hotkey register/unregister, permissions check)
        // SCRIBE-5: register onboarding commands (open/close, is_complete, mark_complete)
        // SCRIBE-6: register tray commands (idle icon, menu items)
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn scaffold_compiles() {}
}
