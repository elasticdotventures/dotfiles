# Just Command Runner - b00t Gospel

Just is a command runner (not a build system) that avoids make's complexity and idiosyncrasies.

## References
- [Official Just Manual](https://just.systems/man/en/)
- [Working Directory Control](https://just.systems/man/en/working-directory.html)
- [Aliases](https://just.systems/man/en/aliases.html)

## Core Syntax & Environment Variables

### Settings & Environment Loading
```bash
# Essential settings for b00t projects
set dotenv-load          # Load .env files automatically
set export               # Export all just variables as env vars
set shell := ["bash", "-c"]

# Access environment variables with $VAR (NOT {{var}})
serve:
    @echo "Starting server on port $SERVER_PORT..."
    ./server --database $DATABASE_ADDRESS --port $SERVER_PORT
```

### Variables vs Environment Variables
- **Just variables**: `var := "value"` → accessed with `{{var}}`  
- **Environment variables**: From .env or system → accessed with `$VAR`
- **Export just vars**: Use `set export` or `export VAR := "value"`

## Shebang Recipes (Complex Multiline Scripts)

⚠️ **Key Pattern**: For complex logic, file creation, heredocs → Use shebang recipes:

```bash
# Environment file creation (the b00t way)
init:
    #!/usr/bin/env bash
    if [[ ! -f .env ]]; then
        cat > .env << 'EOF'
# Project Environment Configuration  
PROJECT_NAME=my-project
DATABASE_URL=$${DATABASE_URL:-postgresql://localhost/mydb}
API_KEY=your-api-key-here
DEBUG=$${DEBUG:-false}
EOF
        echo "✅ Environment template created"
    else
        echo "✅ Environment file already exists"  
    fi

# Config generation from b00t datums
generate-litellm-config:
    #!/usr/bin/env bash
    echo "# Auto-generated LiteLLM configuration"
    echo "# Generated at: $(date -Iseconds)"
    echo ""
    echo "model_list:"
    
    for file in *.ai_model.toml; do
        [[ -f "$file" ]] || continue
        name=$(basename "$file" .ai_model.toml)
        model=$(toml get "$file" ai_model.litellm_model 2>/dev/null | tr -d '"')
        api_key_env=$(toml get "$file" ai_model.api_key_env 2>/dev/null | tr -d '"')
        
        echo "  - model_name: $name"
        echo "    litellm_params:"
        echo "      model: $model"
        if [[ -n "$api_key_env" ]]; then
            echo "      api_key: \"os.environ/$api_key_env\""
        fi
        echo ""
    done
    
    echo "general_settings:"
    echo "  master_key: \"os.environ/LITELLM_MASTER_KEY\""

# Python with environment access
python-task:
    #!/usr/bin/env python3
    import os
    print(f"Database: {os.environ.get('DATABASE_URL')}")
    # Multi-line Python with full env access
```

## Multi-line Constructs & Heredocs

⚠️ **Problem**: Regular recipes run line-by-line, breaking multiline constructs.

### ❌ Wrong Way (Breaks):
```bash
# This FAILS - justfile parses the heredoc content
bad-example:
    cat > file.txt << 'EOF'
    KEY=value  # justfile tries to parse this!
    EOF
```

### ✅ Right Way (Shebang):
```bash  
# This WORKS - shell handles the heredoc
good-example:
    #!/usr/bin/env bash
    cat > file.txt << 'EOF'
    KEY=value
    DATABASE_URL=$${DATABASE_URL:-default}
    EOF
```

### Alternative (Multiple Commands):
```bash
# Fallback pattern for simple cases
simple-env:
    @echo "# Generated config" > config/.env
    @echo "API_KEY=$${API_KEY:-please-set-me}" >> config/.env
    @echo "DEBUG=$${DEBUG:-false}" >> config/.env
```

## Variable Escaping & Interpolation

```bash
# Just variable interpolation
project := "myapp"
deploy:
    echo "Deploying {{project}}"  # → "Deploying myapp"

# Environment variable (runtime)
deploy:
    echo "User: $USER"            # → "User: brianh"

# Escape just interpolation to pass literal ${VAR} to shell
create-template:
    echo "USER=$${USER}" > .env   # → writes "USER=brianh" to file
    # Note: $${VAR} becomes ${VAR} in shell, then expands to value
```

## b00t Patterns & Best Practices

### 🚩 Security: Environment Templates
```bash
init-secure:
    #!/usr/bin/env bash
    if [[ ! -f .env ]]; then
        # Use single quotes to prevent expansion during creation
        cat > .env << 'EOF'
# LiteLLM Configuration
LITELLM_MASTER_KEY=sk-change-me-$(openssl rand -hex 8)
ANTHROPIC_API_KEY=
OPENAI_API_KEY=
FIREWORKS_API_KEY=
EOF
        chmod 600 .env  # Secure permissions
        echo "🔒 Secure environment template created"
    fi
```

### Model Registration System
```bash
# Register new AI model datum
register-model model_name provider:
    #!/usr/bin/env bash
    cat > "{{model_name}}.ai_model.toml" << EOF
[b00t]
name = "{{model_name}}"
type = "ai_model" 
hint = "{{provider}} model - edit this file"

[ai_model]
provider = "{{provider}}"
size = "large"
capabilities = ["chat"]
litellm_model = "{{provider}}/{{model_name}}"
api_key_env = "$(echo {{provider}} | tr '[:lower:]' '[:upper:]')_API_KEY"
enabled = true
EOF
    echo "✅ Model datum created: {{model_name}}.ai_model.toml"
```

### Complex Conditionals
```bash
ci-setup:
    #!/usr/bin/env bash
    set -euo pipefail
    
    if [[ "${CI:-}" == "true" ]]; then
        echo "🤖 Running in CI environment"
        export RUST_LOG=debug
        export NODE_ENV=test
    else
        echo "🏠 Local development setup"
        export RUST_LOG=info
        export NODE_ENV=development
    fi
    
    # Multi-step setup
    for component in api worker ui; do
        echo "Setting up $component..."
        mkdir -p "logs/$component"
        touch "logs/$component/$(date +%Y-%m-%d).log"
    done
```

## Advanced Features

### Dependencies & Parameters
```bash
# Recipe dependencies
deploy: test build
    kubectl apply -f k8s/

# Parameters with defaults
deploy-to target="staging":
    echo "Deploying to {{target}}"
    kubectl apply -f k8s/{{target}}.yaml

# Environment + parameters
test-model model_name:
    #!/usr/bin/env bash
    curl -X POST http://localhost:4000/chat/completions \
        -H "Authorization: Bearer $LITELLM_MASTER_KEY" \
        -d '{"model": "{{model_name}}", "messages": [{"role": "user", "content": "Hello"}]}'
```

### Error Handling & Prefixes
```bash
# Ignore errors with -
optional-cleanup:
    -rm -rf temp/
    echo "Cleanup attempted"

# Suppress output with @  
quiet-task:
    @echo "This is shown"
    @-failing-command  # Fails silently

# Both prefixes
really-quiet:
    @-rm -rf temp/ 2>/dev/null
```

## Working Directory Control

### Default Behavior
```bash
# By default, recipes run in the justfile's directory
default:
    pwd  # → /path/to/justfile/directory
```

### Control Working Directory
```bash
# Global working directory override  
set working-directory := 'src'

# Recipe runs in invocation directory (where just was called)
[no-cd]
build:
    pwd  # → directory where `just build` was executed

# Specific working directory for a recipe
[working-directory: '/tmp']
temp-task:
    pwd  # → /tmp
```

### Path Functions
```bash
# Just provides path functions for file locations
config:
    echo "Justfile at: {{justfile_directory()}}"
    echo "Parent dir: {{parent_directory(justfile_directory())}}"
    echo "Invocation dir: {{invocation_directory()}}"
```

## Aliases & Shortcuts

### Basic Aliases
```bash
# Create shorthand commands
alias b := build
alias t := test
alias dev := start-dev-server

build:
    cargo build --release

test:
    cargo test

start-dev-server:
    npm run dev
```

### Module Aliases
```bash
# Reference recipes in submodules
mod frontend
mod backend

alias ui := frontend::start
alias api := backend::serve
```

## Key Rules for b00t

1. **Use shebang recipes** for any complex logic, file creation, or heredocs
2. **Environment variables** use `$VAR` syntax (not `{{var}}`)
3. **Security first**: Use templates for sensitive configs, never hardcode keys
4. **Escape interpolation** with `$${VAR}` when passing to shell
5. **Keep simple recipes simple**: Single commands don't need shebangs
6. **Load environment**: Always use `set dotenv-load` in b00t projects
7. **Use aliases** for frequently used commands: `alias d := deploy`
8. **Control working directory** with `[no-cd]` or `set working-directory`

### ⚠️ Common Gotchas

- `#!/usr/bin/env bash` (space after #!)  
- Heredocs only work in shebang recipes
- `$${VAR}` for shell expansion vs `{{var}}` for just variables
- Line-by-line execution breaks multiline shell constructs
- Environment files need secure permissions (`chmod 600`)
- **Working directory**: Recipes default to justfile's directory, not invocation directory
- **Path functions**: Use `{{justfile_directory()}}` not relative paths in file operations

🤓 **Remember**: Just is a command runner, not a shell script. For shell logic, use shebang recipes!