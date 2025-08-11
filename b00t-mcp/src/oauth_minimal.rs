use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json, Redirect},
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

// Minimal OAuth configuration for MVP
#[derive(Clone)]
pub struct MinimalOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub jwt_secret: Vec<u8>,
    pub token_lifetime: Duration,
}

impl Default for MinimalOAuthConfig {
    fn default() -> Self {
        Self {
            client_id: "b00t-mcp-client".to_string(),
            client_secret: "b00t-mcp-secret".to_string(),
            jwt_secret: b"your-256-bit-secret-key-change-me!".to_vec(),
            token_lifetime: Duration::from_secs(3600), // 1 hour
        }
    }
}

// Simple auth session storage
#[derive(Clone)]
pub struct MinimalOAuthState {
    pub config: MinimalOAuthConfig,
    pub sessions: Arc<RwLock<HashMap<String, String>>>, // session_id -> user_data
}

impl MinimalOAuthState {
    pub fn new(config: MinimalOAuthConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn generate_access_token(&self, user_id: &str) -> Result<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let exp = now + self.config.token_lifetime.as_secs();

        #[derive(Serialize)]
        struct Claims {
            sub: String,
            aud: String,
            iss: String,
            exp: u64,
            iat: u64,
        }

        let claims = Claims {
            sub: user_id.to_string(),
            aud: self.config.client_id.clone(),
            iss: "b00t-mcp".to_string(),
            exp,
            iat: now,
        };

        let header = Header::default();
        let key = EncodingKey::from_secret(&self.config.jwt_secret);
        
        encode(&header, &claims, &key)
            .map_err(|e| anyhow::anyhow!("Failed to encode JWT: {}", e))
    }
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TokenRequestForm {
    pub grant_type: String,
    pub code: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponseJson {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponseJson {
    pub error: String,
    pub error_description: String,
}

// Minimal OAuth router
pub fn minimal_oauth_router(state: MinimalOAuthState) -> Router {
    Router::new()
        .route("/.well-known/oauth-authorization-server", get(discovery_handler))
        .route("/oauth/authorize", get(authorize_handler))
        .route("/oauth/token", post(token_handler))
        .route("/oauth/consent", get(consent_form_handler).post(consent_post_handler))
        .with_state(state)
}

// Discovery endpoint
async fn discovery_handler(State(state): State<MinimalOAuthState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "issuer": "https://b00t-mcp.local",
        "authorization_endpoint": "https://b00t-mcp.local/oauth/authorize",
        "token_endpoint": "https://b00t-mcp.local/oauth/token",
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "code_challenge_methods_supported": ["plain"],
        "scopes_supported": ["b00t:read", "b00t:write"],
    }))
}

// Authorization endpoint  
async fn authorize_handler(
    State(state): State<MinimalOAuthState>,
    Query(req): Query<AuthRequest>,
) -> Result<Redirect, (StatusCode, String)> {
    // Validate client ID
    if req.client_id != state.config.client_id {
        return Err((StatusCode::BAD_REQUEST, "Invalid client_id".to_string()));
    }

    // Create session
    let session_id = Uuid::new_v4().to_string();
    let redirect_data = format!("{}|{}", req.redirect_uri, req.state.unwrap_or_default());
    
    state.sessions.write().unwrap().insert(session_id.clone(), redirect_data);

    // Redirect to consent
    Ok(Redirect::to(&format!("/oauth/consent?session_id={}", session_id)))
}

// Consent form
async fn consent_form_handler(
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>b00t-mcp Authorization</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; text-align: center; }}
        .container {{ max-width: 400px; margin: 0 auto; }}
        button {{ padding: 10px 20px; margin: 10px; font-size: 16px; cursor: pointer; }}
        .allow {{ background: #4CAF50; color: white; border: none; }}
        .deny {{ background: #f44336; color: white; border: none; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>🥾 b00t-mcp Authorization</h2>
        <p>Claude is requesting access to your b00t tools.</p>
        
        <form method="post" action="/oauth/consent">
            <input type="hidden" name="session_id" value="{}" />
            <button type="submit" name="action" value="allow" class="allow">
                ✅ Allow Access
            </button>
            <button type="submit" name="action" value="deny" class="deny">
                ❌ Deny Access
            </button>
        </form>
    </div>
</body>
</html>
    "#, session_id);

    Html(html)
}

// Consent handler
#[derive(Debug, Deserialize)]
struct ConsentRequest {
    session_id: String,
    action: String,
}

async fn consent_post_handler(
    State(state): State<MinimalOAuthState>,
    axum::extract::Form(form): axum::extract::Form<ConsentRequest>,
) -> Result<Redirect, (StatusCode, String)> {
    let mut sessions = state.sessions.write().unwrap();
    let session_data = sessions.remove(&form.session_id)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Invalid session".to_string()))?;

    let parts: Vec<&str> = session_data.split('|').collect();
    let redirect_uri = parts.get(0).unwrap_or(&"");
    let state_param = parts.get(1).unwrap_or(&"");

    if form.action == "deny" {
        let url = format!("{}?error=access_denied&state={}", redirect_uri, state_param);
        return Ok(Redirect::to(&url));
    }

    // Generate authorization code
    let auth_code = URL_SAFE_NO_PAD.encode(Uuid::new_v4().as_bytes());
    
    // Store code for token exchange (reuse sessions map)
    sessions.insert(auth_code.clone(), "user123".to_string());

    let url = format!("{}?code={}&state={}", redirect_uri, auth_code, state_param);
    Ok(Redirect::to(&url))
}

// Token endpoint
async fn token_handler(
    State(state): State<MinimalOAuthState>,
    axum::extract::Form(req): axum::extract::Form<TokenRequestForm>,
) -> Result<Json<TokenResponseJson>, (StatusCode, Json<ErrorResponseJson>)> {
    // Validate client
    if req.client_id != state.config.client_id || req.client_secret != state.config.client_secret {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponseJson {
                error: "invalid_client".to_string(),
                error_description: "Invalid client credentials".to_string(),
            }),
        ));
    }

    // Validate grant type
    if req.grant_type != "authorization_code" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponseJson {
                error: "unsupported_grant_type".to_string(),
                error_description: "Only authorization_code supported".to_string(),
            }),
        ));
    }

    // Exchange code for token
    let mut sessions = state.sessions.write().unwrap();
    let user_id = sessions.remove(&req.code)
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponseJson {
                error: "invalid_grant".to_string(),
                error_description: "Invalid authorization code".to_string(),
            }),
        ))?;

    // Generate access token
    let access_token = state.generate_access_token(&user_id)
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponseJson {
                error: "server_error".to_string(),
                error_description: "Failed to generate token".to_string(),
            }),
        ))?;

    Ok(Json(TokenResponseJson {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.token_lifetime.as_secs(),
    }))
}