#!/usr/bin/env bash
# Cross-platform environment setup script for LLM Terminal
# Works with bash, zsh, and other POSIX-compliant shells
# Usage: source setup_env.sh

echo "🚀 LLM Terminal Environment Setup"
echo "================================="

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to prompt for API key
prompt_for_api_key() {
    local service="$1"
    local var_name="$2"
    local current_value="${!var_name}"
    
    echo ""
    echo "📝 Setting up $service API Key"
    echo "Current value: ${current_value:-"(not set)"}"
    echo "Enter your $service API key (or press Enter to skip):"
    read -s api_key
    
    if [ -n "$api_key" ]; then
        export "$var_name"="$api_key"
        echo "✅ $service API key set successfully"
    elif [ -n "$current_value" ]; then
        echo "ℹ️  Using existing $service API key"
    else
        echo "⚠️  $service API key not set - this provider will be unavailable"
    fi
}

# Check if Rust/Cargo is installed
if ! command_exists cargo; then
    echo "❌ Cargo not found. Please install Rust from https://rustup.rs/"
    return 1 2>/dev/null || exit 1
fi

# Check existing environment variables
echo ""
echo "🔍 Checking existing environment variables..."

# Anthropic Claude API Key (check both common variable names)
if [ -n "$ANTHROPIC_API_KEY" ]; then
    echo "✅ ANTHROPIC_API_KEY is set"
elif [ -n "$CLAUDE_API_KEY" ]; then
    echo "✅ CLAUDE_API_KEY is set"
    export ANTHROPIC_API_KEY="$CLAUDE_API_KEY"
else
    echo "⚠️  No Claude API key found"
fi

# OpenAI API Key
if [ -n "$OPENAI_API_KEY" ]; then
    echo "✅ OPENAI_API_KEY is set"
else
    echo "⚠️  No OpenAI API key found"
fi

# Interactive setup if no keys are found
if [ -z "$ANTHROPIC_API_KEY" ] && [ -z "$OPENAI_API_KEY" ]; then
    echo ""
    echo "🔑 No API keys found. Let's set them up interactively."
    echo "You can skip any provider you don't want to use."
    
    prompt_for_api_key "Anthropic Claude" "ANTHROPIC_API_KEY"
    prompt_for_api_key "OpenAI" "OPENAI_API_KEY"
fi

# Build the application
echo ""
echo "🔨 Building LLM Terminal..."
if cargo build --release; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    return 1 2>/dev/null || exit 1
fi

# Create platform-specific executable path
EXECUTABLE_PATH="./target/release/llm-terminal"
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    EXECUTABLE_PATH="${EXECUTABLE_PATH}.exe"
fi

# Display setup summary
echo ""
echo "📋 Setup Summary"
echo "================"
echo "Executable: $EXECUTABLE_PATH"
echo "Anthropic Claude: ${ANTHROPIC_API_KEY:+✅ Configured}${ANTHROPIC_API_KEY:-❌ Not configured}"
echo "OpenAI: ${OPENAI_API_KEY:+✅ Configured}${OPENAI_API_KEY:-❌ Not configured}"

# Function to run the application
run_llm_terminal() {
    echo ""
    echo "🚀 Starting LLM Terminal..."
    echo "Press Ctrl+Q to quit, Ctrl+, for settings"
    echo ""
    "$EXECUTABLE_PATH"
}

# Export the function so it can be called from the shell
export -f run_llm_terminal 2>/dev/null || true

echo ""
echo "🎉 Setup complete! Run 'run_llm_terminal' or '${EXECUTABLE_PATH}' to start."
echo ""
echo "💡 Quick start commands:"
echo "  run_llm_terminal    # Start the application"
echo "  cargo run           # Run from source"
echo "  cargo test          # Run tests"
echo ""
echo "🔧 Keyboard shortcuts in the application:"
echo "  Ctrl+Q or Ctrl+C   # Quit"
echo "  Ctrl+T              # New tab"
echo "  Ctrl+W              # Close tab"
echo "  Ctrl+,              # Settings panel"
echo "  Tab/Shift+Tab       # Switch tabs"
echo "  Enter               # Send message"
