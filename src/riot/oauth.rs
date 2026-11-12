use crate::{anyhow, bail, error::Result, globals::CLIENT, riot::settings::SavedTokens};
use serde_json::Value;
use std::collections::HashMap;

const OAUTH_TOKEN_URL: &str = "https://auth.riotgames.com/token";
const AUTH_SESSION_URL: &str = "https://auth.riotgames.com/api/v1/authorization";

/// Exchange saved Riot Client tokens for a fresh `(access_token, id_token)` pair.
///
/// Tries the OAuth2 `refresh_token` grant first, then falls back to cookie-based
/// re-authentication using the saved `tdid` cookie.
pub fn exchange_tokens(saved: &SavedTokens) -> Result<(String, String)> {
    if let Some(tokens) = try_refresh_grant(&saved.refresh_token, &saved.id_token) {
        return Ok(tokens);
    }

    reauth_with_cookies(saved)
}

fn try_refresh_grant(refresh_token: &str, fallback_id_token: &str) -> Option<(String, String)> {
    let form_body = format!(
        "grant_type=refresh_token&client_id=riot-client&refresh_token={}",
        refresh_token
    );

    let mut resp = CLIENT
        .post(OAUTH_TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(&form_body)
        .ok()?;

    let json: Value = resp.body_mut().read_json().ok()?;
    let access_token = json["access_token"].as_str()?;
    let id_token = json["id_token"].as_str().unwrap_or(fallback_id_token);

    Some((access_token.to_string(), id_token.to_string()))
}

fn reauth_with_cookies(saved: &SavedTokens) -> Result<(String, String)> {
    let auth_body = serde_json::json!({
        "client_id": "play-valorant-web-prod",
        "nonce": "1",
        "redirect_uri": "https://playvalorant.com/opt_in",
        "response_type": "token id_token",
        "scope": "account openid"
    });

    let mut cookies = Vec::new();
    if let Some(tdid) = &saved.tdid {
        cookies.push(format!("tdid={}", tdid));
    }

    // Step 1: Initialize an auth session and capture the `ssid` cookie if issued.
    if let Some(ssid) = fetch_ssid_cookie(&cookies.join("; "), &auth_body) {
        cookies.push(ssid);
    }

    // Step 2: Re-submit with cookies so Riot returns the redirect URI containing tokens.
    let cookie_header = cookies.join("; ");
    let mut resp = CLIENT
        .post(AUTH_SESSION_URL)
        .header("Cookie", cookie_header.as_str())
        .header("Content-Type", "application/json")
        .send_json(&auth_body)
        .map_err(|_| anyhow!("Failed to contact Riot auth server"))?;

    let json: Value = resp
        .body_mut()
        .read_json()
        .map_err(|_| anyhow!("Invalid response from Riot auth server"))?;

    if let Some(tokens) = extract_tokens_from_response(&json, &saved.id_token) {
        return Ok(tokens);
    }

    bail!("Your session has expired. Please log in again and save your session to continue.")
}

fn fetch_ssid_cookie(cookie_header: &str, auth_body: &Value) -> Option<String> {
    let resp = CLIENT
        .post(AUTH_SESSION_URL)
        .header("Content-Type", "application/json")
        .header("Cookie", cookie_header)
        .send_json(auth_body)
        .ok()?;

    for header in resp.headers().get_all("set-cookie") {
        if let Ok(value) = header.to_str() {
            if value.starts_with("ssid=") {
                if let Some(ssid_pair) = value.split(';').next() {
                    return Some(ssid_pair.to_string());
                }
            }
        }
    }

    None
}

fn extract_tokens_from_response(
    resp: &Value,
    fallback_id_token: &str,
) -> Option<(String, String)> {
    if resp["type"].as_str() != Some("response") {
        return None;
    }

    let uri = resp["response"]["parameters"]["uri"].as_str()?;
    parse_redirect_fragment(uri, fallback_id_token)
}

fn parse_redirect_fragment(uri: &str, fallback_id_token: &str) -> Option<(String, String)> {
    let fragment = uri.split('#').nth(1)?;
    let params: HashMap<&str, &str> = fragment
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .collect();

    let access_token = params.get("access_token")?;
    let id_token = params
        .get("id_token")
        .copied()
        .unwrap_or(fallback_id_token);

    Some(((*access_token).to_string(), id_token.to_string()))
}

