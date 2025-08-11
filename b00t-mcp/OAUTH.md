# 🔐 OAuth 2.1 Authorization with b00t-mcp

This document explains how to authenticate against the b00t-mcp server using OAuth 2.1 authorization code flow.

## Quick Start

1. **Start the server**:
```bash
cargo run --release -p b00t-mcp -- --http --port 8080
```

2. **Server endpoints**:
- MCP: `http://127.0.0.1:8080/mcp`
- OAuth Discovery: `http://127.0.0.1:8080/.well-known/oauth-authorization-server`

## OAuth 2.1 Flow

### Step 1: Discovery (Optional)

Get server OAuth configuration:

```bash
curl http://127.0.0.1:8080/.well-known/oauth-authorization-server
```

Response:
```json
{
  "issuer": "https://b00t-mcp.local",
  "authorization_endpoint": "https://b00t-mcp.local/oauth/authorize",
  "token_endpoint": "https://b00t-mcp.local/oauth/token",
  "response_types_supported": ["code"],
  "grant_types_supported": ["authorization_code"],
  "scopes_supported": ["b00t:read", "b00t:write"]
}
```

### Step 2: Authorization Request

Redirect user to authorization endpoint:

```
http://127.0.0.1:8080/oauth/authorize?client_id=b00t-mcp-client&redirect_uri=https://claude.ai/oauth/callback&state=random123&response_type=code
```

**Parameters**:
- `client_id`: `b00t-mcp-client` (default)
- `redirect_uri`: Where to redirect after authorization
- `state`: Random string for security
- `response_type`: Must be `code`

### Step 3: User Consent

User sees consent form and clicks **"✅ Allow Access"** or **"❌ Deny Access"**.

### Step 4: Authorization Code

On approval, user redirected to:
```
https://claude.ai/oauth/callback?code=AUTHORIZATION_CODE&state=random123
```

### Step 5: Token Exchange

Exchange authorization code for access token:

```bash
curl -X POST http://127.0.0.1:8080/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=AUTHORIZATION_CODE" \
  -d "client_id=b00t-mcp-client" \
  -d "client_secret=b00t-mcp-secret"
```

Response:
```json
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer", 
  "expires_in": 3600
}
```

### Step 6: Use Access Token

Include token in MCP requests:
```bash
curl http://127.0.0.1:8080/mcp/some-endpoint \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

## Client Configuration

### Default Client Credentials
- **Client ID**: `b00t-mcp-client`
- **Client Secret**: `b00t-mcp-secret`
- **Redirect URI**: `https://claude.ai/oauth/callback`

### Supported Scopes
- `b00t:read`: Read access to b00t tools and status
- `b00t:write`: Execute b00t commands (safe operations)

## Integration Examples

### Claude Custom Connector

Configure Claude with:
```json
{
  "mcp_server_url": "http://127.0.0.1:8080/mcp",
  "oauth": {
    "authorization_url": "http://127.0.0.1:8080/oauth/authorize",
    "token_url": "http://127.0.0.1:8080/oauth/token",
    "client_id": "b00t-mcp-client",
    "client_secret": "b00t-mcp-secret",
    "scopes": ["b00t:read", "b00t:write"]
  }
}
```

### Manual Testing with curl

1. **Get authorization code** (browser):
   - Visit: `http://127.0.0.1:8080/oauth/authorize?client_id=b00t-mcp-client&redirect_uri=http://localhost&state=test123&response_type=code`
   - Click "Allow Access"
   - Copy code from redirect URL

2. **Exchange for token**:
```bash
curl -X POST http://127.0.0.1:8080/oauth/token \
  -d "grant_type=authorization_code&code=YOUR_CODE&client_id=b00t-mcp-client&client_secret=b00t-mcp-secret"
```

3. **Test MCP with token**:
```bash
curl http://127.0.0.1:8080/mcp \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

## Security Notes

🔐 **Current Implementation**:
- JWT tokens with 1-hour expiration
- Simple consent flow
- Single-user system ("user123")
- In-memory session storage

⚠️ **Production Considerations**:
- Use HTTPS in production
- Implement proper user management
- Add token refresh capability
- Use persistent session storage
- Configure proper CORS policies
- Rotate JWT signing keys

## Troubleshooting

### Common Issues

**Invalid client_id**: 
- Ensure using `b00t-mcp-client`

**Invalid redirect_uri**:
- Must match registered URI exactly
- Default: `https://claude.ai/oauth/callback`

**Expired authorization code**:
- Codes are single-use and expire quickly
- Get fresh code from authorization endpoint

**Invalid token**:
- Tokens expire in 1 hour
- Check JWT expiration with decoder

### Debug Mode

Start server with verbose logging:
```bash
RUST_LOG=debug cargo run --release -p b00t-mcp -- --http --port 8080
```

## Next Steps

🚀 **For Production Deployment**:
1. Deploy to Cloudflare Workers/Container
2. Configure proper SSL certificates
3. Implement user registration/management
4. Add token refresh flow
5. Set up monitoring and logging

📋 **Current Status**: OAuth 2.1 MVP complete, ready for Anthropic Custom Connector integration!