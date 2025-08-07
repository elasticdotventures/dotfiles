# LiteLLM Integration - b00t Gospel

LiteLLM provides a unified proxy server for multiple AI providers through OpenAI-compatible API endpoints.

## References
- [LiteLLM Documentation](https://docs.litellm.ai/)
- [LiteLLM GitHub](https://github.com/BerriAI/litellm)
- [b00t Issue #36](https://github.com/elasticdotventures/dotfiles/issues/36)

## Architecture

### b00t AI Model Datums
AI model configurations are stored as TOML files using b00t's datum pattern:

```toml
# claude-3-5-sonnet.ai_model.toml
[b00t]
name = "claude-3-5-sonnet"
type = "ai_model"
hint = "Anthropic Claude 3.5 Sonnet - premium reasoning model"

[ai_model]
provider = "anthropic"
size = "large"
capabilities = ["chat", "vision", "reasoning"]
litellm_model = "anthropic/claude-3-5-sonnet-20241022"
api_key_env = "ANTHROPIC_API_KEY"
api_base = ""
rpm_limit = 60
enabled = true
```

### Configuration Generation
The `_scan-models` recipe automatically discovers all `*.ai_model.toml` files and generates LiteLLM-compatible YAML:

```yaml
model_list:
  - model_name: claude-3-5-sonnet
    litellm_params:
      model: anthropic/claude-3-5-sonnet-20241022
      api_key: "os.environ/ANTHROPIC_API_KEY"
      rpm: 60
```

## b00t Justfile Commands

### Core Operations
```bash
# Quick start with aliases
just --justfile litellm.just s          # Start proxy (alias)
just --justfile litellm.just start      # Start proxy (alias)
just --justfile litellm.just up         # Start proxy (alias)
just --justfile litellm.just run        # Start proxy (full command)

# Management
just --justfile litellm.just stop       # Stop proxy
just --justfile litellm.just restart    # Restart proxy
just --justfile litellm.just logs       # View logs
```

### Configuration & Discovery
```bash
# Setup and configuration
just --justfile litellm.just init       # Create .env template
just --justfile litellm.just config     # Generate config from datums
just --justfile litellm.just models     # List available models

# Status and testing
just --justfile litellm.just status     # Show proxy status
just --justfile litellm.just test       # Health check + completion test
```

### Maintenance
```bash
just --justfile litellm.just clean      # Clean up containers and files
```

## Environment Configuration

### Security Pattern
API keys are stored in environment variables, never hardcoded:

```bash
# litellm/.env (created by `just init`)
LITELLM_MASTER_KEY=sk-b00t-dev-key-please-change-me

# Provider API Keys
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
FIREWORKS_API_KEY=...
GROQ_API_KEY=gsk_...
```

### Container Configuration
```bash
# Uses podman with environment file mounting
podman run --replace --name litellm-proxy \
    --rm -d \
    -v ./litellm/_b00t_generated.yaml:/app/config.yaml \
    -p 4000:4000 \
    --env-file ./litellm/.env \
    ghcr.io/berriai/litellm:main-latest \
    --config /app/config.yaml --detailed_debug
```

## Working Directory Patterns

### Just Recipe Execution
Recipes run in the justfile's directory by default, enabling simple path resolution:

```bash
# ✅ Correct: Uses justfile working directory
_scan-models:
    #!/usr/bin/env bash
    model_files=(*.ai_model.toml)  # Finds files in same directory
```

### Path Functions
```bash
# Available Just path functions
echo "Justfile: {{justfile_directory()}}"      # /path/to/_b00t_
echo "Parent: {{parent_directory(justfile_directory())}}"  # /path/to
echo "Invocation: {{invocation_directory()}}"  # Where just was called
```

## Integration Workflow

### 1. Model Registration
Create AI model datums in `_b00t_/` directory:

```bash
# Manual creation
cat > gpt-4o.ai_model.toml << EOF
[b00t]
name = "gpt-4o"
type = "ai_model"
hint = "OpenAI GPT-4o - multimodal flagship model"

[ai_model]
provider = "openai"
size = "large"
capabilities = ["chat", "vision", "function_calling"]
litellm_model = "openai/gpt-4o"
api_key_env = "OPENAI_API_KEY"
rpm_limit = 120
enabled = true
EOF
```

### 2. Configuration Generation
```bash
# Generate LiteLLM config from all datums
just --justfile litellm.just config
```

### 3. Proxy Startup
```bash
# Initialize environment (first time)
just --justfile litellm.just init

# Edit API keys in litellm/.env
# Start proxy
just --justfile litellm.just s
```

### 4. Testing & Validation
```bash
# Health check
just --justfile litellm.just test

# Manual API test
curl -X POST http://localhost:4000/chat/completions \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $LITELLM_MASTER_KEY" \
    -d '{"model": "claude-3-5-sonnet", "messages": [{"role": "user", "content": "Hello!"}]}'
```

## b00t Patterns Applied

### 1. Datum Architecture
- **Abstract Schema**: `datum_ai_model.rs` defines structure
- **TOML Data**: Individual model configurations in files
- **No Recompilation**: Add/remove models without code changes

### 2. Security First
- **Environment Variables**: No hardcoded API keys
- **Template Generation**: Safe .env creation
- **Permissions**: Secure file permissions (`chmod 600`)

### 3. Tool Integration
- **Container Runtime**: Uses podman (b00t standard)
- **Configuration Management**: Just recipes for automation
- **Path Resolution**: Proper working directory handling

### 4. Operational Excellence
- **Health Checks**: Automated proxy testing
- **Log Access**: Easy log viewing with `logs` command
- **Clean Shutdown**: Proper container lifecycle management

## Common Patterns

### Model Size Classification
```toml
# sm0l models (fast, efficient)
[ai_model]
size = "small"
rpm_limit = 1000

# ch0nky models (powerful, expensive)  
[ai_model]
size = "large"
rpm_limit = 60
```

### Provider-Specific Configuration
```toml
# Azure OpenAI requires additional config
[ai_model]
provider = "azure"
litellm_model = "azure/gpt-4o"
api_key_env = "AZURE_API_KEY"
api_base = "https://your-endpoint.openai.azure.com/"
```

### Capability Tagging
```toml
[ai_model]
capabilities = ["chat", "vision", "function_calling", "reasoning"]
```

## Troubleshooting

### Path Resolution Issues
```bash
# ❌ Wrong: Trying to escape justfile directory
model_files=({{parent_directory(justfile_directory())}}/*.toml)

# ✅ Right: Use justfile's working directory
model_files=(*.ai_model.toml)
```

### Recipe Dependencies
```bash
# ❌ Wrong: Direct recipe calls fail
just _scan-models > config.yaml

# ✅ Right: Specify justfile explicitly  
just --justfile {{justfile()}} _scan-models > config.yaml
```

### Container Issues
```bash
# Check container status
podman ps --filter name=litellm-proxy

# View container logs
podman logs litellm-proxy

# Clean restart
just --justfile litellm.just clean
just --justfile litellm.just run
```

## Future Enhancements

### Provider Registration System
- Dynamic model discovery from provider APIs
- Automatic capability detection
- Cost optimization routing

### MCP Integration
- b00t-mcp server for model management
- Claude Code integration
- Agent-driven model selection

### Monitoring & Analytics
- Usage tracking per model
- Cost analysis
- Performance metrics

🤓 **Remember**: LiteLLM is a proxy, not a model host. You still need valid API keys for each provider you configure.