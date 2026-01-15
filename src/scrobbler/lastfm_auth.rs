// Last.fm authentication helper using token-based flow

use anyhow::{Context, Result};
use attohttpc;
use rustfm_scrobble_proxy::Scrobbler;
use serde::Deserialize;

const LASTFM_API_URL: &str = "https://ws.audioscrobbler.com/2.0/";
const LASTFM_AUTH_URL: &str = "https://www.last.fm/api/auth/";

#[derive(Debug, Deserialize)]
struct LastFmResponse {
    token: Option<String>,
}

/// Get an authentication token from Last.fm
fn get_token(api_key: &str, api_secret: &str) -> Result<String> {
    // Create API signature for getToken request
    let sig_string = format!("api_key{}method{}{}", api_key, "auth.gettoken", api_secret);
    let signature = format!("{:x}", md5::compute(sig_string.as_bytes()));

    // Build form-encoded body
    let body = format!(
        "method=auth.gettoken&api_key={}&api_sig={}&format=json",
        api_key, signature
    );

    let response = attohttpc::post(LASTFM_API_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .text(body)
        .send()
        .context("Failed to get token from Last.fm")?;

    if !response.is_success() {
        anyhow::bail!("Last.fm API error: {}", response.status());
    }

    let data: LastFmResponse = response.json()?;
    data.token
        .ok_or_else(|| anyhow::anyhow!("No token in Last.fm response"))
}

/// Perform the complete Last.fm authentication flow using token-based auth
/// Returns the session key on success
pub fn authenticate(api_key: &str, api_secret: &str) -> Result<String> {
    println!("Starting Last.fm authentication...\n");

    // Step 1: Get authentication token
    println!("Getting authorization token...");
    let token = get_token(api_key, api_secret)?;
    println!("Token obtained: {}\n", token);

    // Step 2: Direct user to authorize
    let auth_url = format!("{}?api_key={}&token={}", LASTFM_AUTH_URL, api_key, token);
    println!("Please authorize this application:");
    println!("  {}\n", auth_url);
    println!("Opening authorization URL in your browser...");

    // Try to open the URL in the default browser
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&auth_url)
            .spawn();
    }

    println!("\nAfter authorizing, press Enter to continue...");

    // Wait for user to press Enter
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Step 3: Exchange token for session key
    println!("\nExchanging token for session key...");
    let mut scrobbler = Scrobbler::new(api_key, api_secret);
    let session = scrobbler.authenticate_with_token(&token)?;
    println!("Session key obtained successfully!\n");

    Ok(session.key)
}
