// Last.fm authentication helper
// Implements the authentication flow to obtain a session key

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeMap;

const LASTFM_API_URL: &str = "https://ws.audioscrobbler.com/2.0/";
const LASTFM_AUTH_URL: &str = "https://www.last.fm/api/auth/";

#[derive(Debug, Deserialize)]
struct LastFmResponse {
    token: Option<String>,
    session: Option<Session>,
}

#[derive(Debug, Deserialize)]
struct Session {
    key: String,
}

/// Generate API signature for Last.fm requests
fn generate_signature(params: &BTreeMap<String, String>, api_secret: &str) -> String {
    let mut sig_string = String::new();
    for (key, value) in params.iter() {
        sig_string.push_str(key);
        sig_string.push_str(value);
    }
    sig_string.push_str(api_secret);

    format!("{:x}", md5::compute(sig_string.as_bytes()))
}

/// Get a request token from Last.fm
async fn get_token(api_key: &str, api_secret: &str) -> Result<String> {
    let client = Client::new();

    let mut params = BTreeMap::new();
    params.insert("method".to_string(), "auth.getToken".to_string());
    params.insert("api_key".to_string(), api_key.to_string());

    let signature = generate_signature(&params, api_secret);
    params.insert("api_sig".to_string(), signature);
    params.insert("format".to_string(), "json".to_string());

    let response = client
        .post(LASTFM_API_URL)
        .form(&params)
        .send()
        .await
        .context("Failed to get token from Last.fm")?;

    if !response.status().is_success() {
        anyhow::bail!("Last.fm API error: {}", response.status());
    }

    let data: LastFmResponse = response.json().await?;

    data.token
        .ok_or_else(|| anyhow::anyhow!("No token in Last.fm response"))
}

/// Exchange token for session key
async fn get_session(api_key: &str, api_secret: &str, token: &str) -> Result<String> {
    let client = Client::new();

    let mut params = BTreeMap::new();
    params.insert("method".to_string(), "auth.getSession".to_string());
    params.insert("api_key".to_string(), api_key.to_string());
    params.insert("token".to_string(), token.to_string());

    let signature = generate_signature(&params, api_secret);
    params.insert("api_sig".to_string(), signature);
    params.insert("format".to_string(), "json".to_string());

    let response = client
        .post(LASTFM_API_URL)
        .form(&params)
        .send()
        .await
        .context("Failed to get session from Last.fm")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Last.fm API error ({}): {}", status, body);
    }

    let data: LastFmResponse = response.json().await?;

    data.session
        .map(|s| s.key)
        .ok_or_else(|| anyhow::anyhow!("No session key in Last.fm response"))
}

/// Perform the complete Last.fm authentication flow
pub async fn authenticate(api_key: &str, api_secret: &str) -> Result<String> {
    println!("Starting Last.fm authentication...\n");

    // Step 1: Get token
    println!("Getting authorization token...");
    let token = get_token(api_key, api_secret).await?;
    println!("Token obtained: {}\n", token);

    // Step 2: Direct user to authorize
    let auth_url = format!("{}?api_key={}&token={}", LASTFM_AUTH_URL, api_key, token);
    println!("Please authorize this application:");
    println!("  {}\n", auth_url);
    println!("After authorizing, press Enter to continue...");

    // Wait for user to press Enter
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Step 3: Get session key
    println!("\nExchanging token for session key...");
    let session_key = get_session(api_key, api_secret, &token).await?;
    println!("Session key obtained successfully!\n");

    Ok(session_key)
}
