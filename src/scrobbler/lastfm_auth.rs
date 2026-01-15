// Last.fm authentication helper using rustfm-scrobble-proxy

use anyhow::Result;
use rustfm_scrobble_proxy::Scrobbler;
use std::io::{self, Write};

/// Perform the complete Last.fm authentication flow using username/password
/// Returns the session key on success
pub fn authenticate(api_key: &str, api_secret: &str) -> Result<String> {
    println!("Starting Last.fm authentication...\n");

    let mut scrobbler = Scrobbler::new(api_key, api_secret);

    // Get username from user
    print!("Last.fm Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    username = username.trim().to_string();

    // Get password from user (hidden input would be better, but this is simple)
    print!("Last.fm Password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    password = password.trim().to_string();

    // Authenticate and get session
    println!("\nAuthenticating with Last.fm...");
    let session = scrobbler.authenticate_with_password(&username, &password)?;
    println!("Session key obtained successfully!\n");

    Ok(session.key)
}
