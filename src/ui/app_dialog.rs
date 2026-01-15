// App filtering dialog using NSAlert

use objc2_app_kit::{NSAlert, NSAlertFirstButtonReturn, NSAlertSecondButtonReturn, NSAlertStyle};
use objc2_foundation::{MainThreadMarker, NSString};

/// User's choice for an app
#[derive(Debug, PartialEq)]
pub enum AppChoice {
    Allow,
    Ignore,
}

/// Show a native macOS alert asking the user whether to allow or ignore scrobbling from an app
pub fn show_app_prompt(bundle_id: &str) -> AppChoice {
    // SAFETY: This function must be called from the main thread
    // The caller (main.rs event loop) ensures this
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    unsafe {
        // Create NSAlert
        let alert = NSAlert::new(mtm);

        // Set alert style to informational
        alert.setAlertStyle(NSAlertStyle::Informational);

        // Set message text
        let message = NSString::from_str("Allow scrobbling from this app?");
        alert.setMessageText(&message);

        // Set informative text with bundle ID
        let info_text = NSString::from_str(&format!(
            "OSX Scrobbler detected music playing from:\n\n{}\n\nWould you like to scrobble from this app?",
            bundle_id
        ));
        alert.setInformativeText(&info_text);

        // Add buttons
        let allow_button = NSString::from_str("Allow");
        let ignore_button = NSString::from_str("Ignore");

        alert.addButtonWithTitle(&allow_button);
        alert.addButtonWithTitle(&ignore_button);

        // Run modal dialog and get response
        let response = alert.runModal();

        // First button (Allow) returns NSAlertFirstButtonReturn
        // Second button (Ignore) returns NSAlertSecondButtonReturn
        if response == NSAlertFirstButtonReturn {
            AppChoice::Allow
        } else if response == NSAlertSecondButtonReturn {
            AppChoice::Ignore
        } else {
            // Default to Ignore for safety if user closes dialog
            AppChoice::Ignore
        }
    }
}
