# LiteLLM Proxy Management - b00t integrated
# Secure, environment-driven proxy server for unified LLM API access

# Aliases for common operations
alias s := run
alias start := run
alias up := run

# Start LiteLLM proxy server with generated configuration
run:
	@echo "🚀 Starting LiteLLM proxy server..."
	@just _ensure-config
	podman run --replace --name litellm-proxy \
		--rm -d \
		-v {{justfile_directory()}}/litellm/_b00t_generated.yaml:/app/config.yaml \
		-p 4000:4000 \
		--env-file {{justfile_directory()}}/litellm/.env \
		ghcr.io/berriai/litellm:main-latest \
		--config /app/config.yaml --detailed_debug
	@echo "✅ LiteLLM proxy started on http://localhost:4000"
	@echo "🌐 Web UI: http://localhost:4000/ui"

# Stop LiteLLM proxy server
stop:
	@echo "🛑 Stopping LiteLLM proxy..."
	-podman stop litellm-proxy
	@echo "✅ LiteLLM proxy stopped"

# Restart proxy (stop + start)
restart:
	@just stop
	@sleep 2
	@just run

# Show proxy logs
logs:
	podman logs -f litellm-proxy

# Test proxy with health check
test:
	@echo "🔍 Testing LiteLLM proxy health..."
	@curl -s http://localhost:4000/health || echo "❌ Proxy not responding"
	@echo ""
	@echo "🧪 Testing model completion..."
	@just _test-completion

# Generate LiteLLM configuration from b00t AI model datums
config:
	@echo "⚙️  Generating LiteLLM config from b00t AI model datums..."
	@just --justfile {{justfile()}} _scan-models > {{justfile_directory()}}/litellm/_b00t_generated.yaml
	@echo "✅ Configuration generated: litellm/_b00t_generated.yaml"

# Show available models from b00t datums
models:
	@echo "📋 Available AI models in b00t registry:"
	@find {{justfile_directory()}} -name "*.ai_model.toml" -exec basename {} .ai_model.toml \; | sort

# Show proxy status and endpoints
status:
	@echo "📊 LiteLLM Proxy Status:"
	@echo "Container: $(podman ps --filter name=litellm-proxy --format 'table {{{{.Names}}}}\t{{{{.Status}}}}\t{{{{.Ports}}}}' 2>/dev/null || echo 'Not running')"
	@echo ""
	@curl -s http://localhost:4000/v1/models 2>/dev/null | jq -r '.data[] | "• " + .id' 2>/dev/null || echo "❌ Proxy not responding"

# Create .env template if it doesn't exist
init:
	@if [ ! -f {{justfile_directory()}}/litellm/.env ]; then \
		echo "🔧 Creating environment template..."; \
		just --justfile {{justfile()}} _create-env-template; \
		echo "✅ Edit litellm/.env with your API keys"; \
	else \
		echo "✅ Environment file already exists"; \
	fi

# Clean up generated files and containers
clean:
	@echo "🧹 Cleaning up LiteLLM resources..."
	-podman rm -f litellm-proxy
	-rm -f {{justfile_directory()}}/litellm/_b00t_generated.yaml
	@echo "✅ Cleanup complete"

# Private: Ensure config exists or generate it
_ensure-config:
	@if [ ! -f {{justfile_directory()}}/litellm/_b00t_generated.yaml ]; then \
		echo "⚙️  Configuration not found, generating..."; \
		just config; \
	fi

# Private: Scan b00t AI model datums and generate litellm YAML
_scan-models:
    #!/usr/bin/env bash
    set -euo pipefail
    
    echo "# Auto-generated LiteLLM configuration from b00t AI model datums"
    echo "# Generated at: $(date -Iseconds)"
    echo ""
    echo "model_list:"
    
    # Scan AI model datums from justfile directory (recipes run here by default)
    model_files=(*.ai_model.toml)
    
    if [[ ! -f "${model_files[0]}" ]]; then
        echo "# No AI model datums found in _b00t_/ directory"
        echo "# Create .ai_model.toml files to configure models"
    else
        for file in "${model_files[@]}"; do
            [[ -f "$file" ]] || continue
            
            name=$(basename "$file" .ai_model.toml)
            echo "  - model_name: $name"
            echo "    litellm_params:"
            
            # Use safer TOML parsing with fallbacks
            if litellm_model=$(toml get "$file" ai_model.litellm_model 2>/dev/null); then
                litellm_model=$(echo "$litellm_model" | tr -d '"')
                echo "      model: $litellm_model"
            else
                echo "      model: openai/$name"
            fi
            
            if api_key_env=$(toml get "$file" ai_model.api_key_env 2>/dev/null); then
                api_key_env=$(echo "$api_key_env" | tr -d '"')
                [[ -n "$api_key_env" ]] && echo "      api_key: \"os.environ/$api_key_env\""
            fi
            
            if api_base=$(toml get "$file" ai_model.api_base 2>/dev/null); then
                api_base=$(echo "$api_base" | tr -d '"')
                [[ -n "$api_base" ]] && echo "      api_base: $api_base"
            fi
            
            if rpm_limit=$(toml get "$file" ai_model.rpm_limit 2>/dev/null); then
                [[ -n "$rpm_limit" && "$rpm_limit" != "null" ]] && echo "      rpm: $rpm_limit"
            fi
            
            echo ""
        done
    fi
    
    echo "general_settings:"
    echo "  master_key: \"os.environ/LITELLM_MASTER_KEY\""
    echo ""
    echo "litellm_settings:"
    echo "  request_timeout: 600"
    echo "  set_verbose: false"
    echo "  json_logs: true"

# Private: Create environment template
_create-env-template:
    #!/usr/bin/env bash
    mkdir -p {{justfile_directory()}}/litellm
    cat > {{justfile_directory()}}/litellm/.env << 'EOF'
    # LiteLLM Environment Configuration
    # 🤓 Add your API keys here - this file is gitignored

    # Master key for LiteLLM proxy authentication
    LITELLM_MASTER_KEY=sk-b00t-dev-key-please-change-me

    # Provider API Keys (uncomment and add your keys)
    # OPENAI_API_KEY=sk-...
    # ANTHROPIC_API_KEY=sk-ant-...
    # AZURE_API_KEY=...
    # GOOGLE_API_KEY=...
    # FIREWORKS_API_KEY=...
    # GROQ_API_KEY=gsk_...
    # XAI_API_KEY=xai_...

    # Provider specific settings
    # AZURE_API_BASE=https://your-azure-endpoint.openai.azure.com/
    # AZURE_API_VERSION=2024-02-15-preview

    # Fireworks (from existing config - REMOVE after migration)  
    # FIREWORKS_ACCOUNT=your-account
    EOF

# Private: Test completion with a simple model
_test-completion:
	@curl -s --location 'http://localhost:4000/chat/completions' \
		--header 'Content-Type: application/json' \
		--header "Authorization: Bearer $$LITELLM_MASTER_KEY" \
		--data '{ \
			"model": "claude-3-5-sonnet", \
			"messages": [{"role": "user", "content": "Hello! Respond with just: b00t integrated ✅"}], \
			"max_tokens": 10 \
		}' | jq -r '.choices[0].message.content' 2>/dev/null || echo "❌ Test failed"