#!/usr/bin/env pwsh
# Cross-platform PowerShell setup script for LLM Terminal
# Works with PowerShell Core 6+ on Windows, macOS, and Linux
# Usage: ./setup_env.ps1

Write-Host "🚀 LLM Terminal Environment Setup" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# Function to check if a command exists
function Test-CommandExists {
    param($Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

# Function to prompt for API key
function Get-ApiKey {
    param(
        [string]$ServiceName,
        [string]$CurrentValue
    )
    
    Write-Host ""
    Write-Host "📝 Setting up $ServiceName API Key" -ForegroundColor Yellow
    Write-Host "Current value: $(if($CurrentValue) { '(set)' } else { '(not set)' })"
    $key = Read-Host "Enter your $ServiceName API key (or press Enter to skip)" -MaskInput
    
    if ($key) {
        Write-Host "✅ $ServiceName API key set successfully" -ForegroundColor Green
        return $key
    } elseif ($CurrentValue) {
        Write-Host "ℹ️  Using existing $ServiceName API key" -ForegroundColor Blue
        return $CurrentValue
    } else {
        Write-Host "⚠️  $ServiceName API key not set - this provider will be unavailable" -ForegroundColor Yellow
        return $null
    }
}

# Check if Rust/Cargo is installed
if (-not (Test-CommandExists "cargo")) {
    Write-Host "❌ Cargo not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🔍 Checking existing environment variables..." -ForegroundColor Blue

# Check existing environment variables
$claudeKey = $env:ANTHROPIC_API_KEY ?? $env:CLAUDE_API_KEY
$openaiKey = $env:OPENAI_API_KEY

if ($claudeKey) {
    Write-Host "✅ Claude API key is set" -ForegroundColor Green
} else {
    Write-Host "⚠️  No Claude API key found" -ForegroundColor Yellow
}

if ($openaiKey) {
    Write-Host "✅ OpenAI API key is set" -ForegroundColor Green
} else {
    Write-Host "⚠️  No OpenAI API key found" -ForegroundColor Yellow
}

# Interactive setup if no keys are found
if (-not $claudeKey -and -not $openaiKey) {
    Write-Host ""
    Write-Host "🔑 No API keys found. Let's set them up interactively." -ForegroundColor Cyan
    Write-Host "You can skip any provider you don't want to use."
    
    $claudeKey = Get-ApiKey "Anthropic Claude" $claudeKey
    $openaiKey = Get-ApiKey "OpenAI" $openaiKey
}

# Set environment variables for this session
if ($claudeKey) { $env:ANTHROPIC_API_KEY = $claudeKey }
if ($openaiKey) { $env:OPENAI_API_KEY = $openaiKey }

# Build the application
Write-Host ""
Write-Host "🔨 Building LLM Terminal..." -ForegroundColor Blue
try {
    cargo build --release
    Write-Host "✅ Build successful!" -ForegroundColor Green
} catch {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

# Determine executable path based on platform
$executableName = if ($IsWindows -or $env:OS -eq "Windows_NT") { "llm-terminal.exe" } else { "llm-terminal" }
$executablePath = Join-Path "target" "release" $executableName

# Display setup summary
Write-Host ""
Write-Host "📋 Setup Summary" -ForegroundColor Cyan
Write-Host "================" -ForegroundColor Cyan
Write-Host "Platform: $($PSVersionTable.Platform ?? 'Windows')" 
Write-Host "Executable: $executablePath"
Write-Host "Anthropic Claude: $(if($env:ANTHROPIC_API_KEY) { '✅ Configured' } else { '❌ Not configured' })"
Write-Host "OpenAI: $(if($env:OPENAI_API_KEY) { '✅ Configured' } else { '❌ Not configured' })"

# Function to run the application
function Start-LlmTerminal {
    Write-Host ""
    Write-Host "🚀 Starting LLM Terminal..." -ForegroundColor Green
    Write-Host "Press Ctrl+Q to quit, Ctrl+, for settings" -ForegroundColor Gray
    Write-Host ""
    & $executablePath
}

# Create alias for easy access
Set-Alias -Name "llm" -Value "Start-LlmTerminal" -Scope Global -Force 2>$null

Write-Host ""
Write-Host "🎉 Setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "💡 Quick start commands:" -ForegroundColor Cyan
Write-Host "  Start-LlmTerminal   # Start the application"
Write-Host "  llm                 # Alias for Start-LlmTerminal"
Write-Host "  cargo run           # Run from source"
Write-Host "  cargo test          # Run tests"
Write-Host ""
Write-Host "🔧 Keyboard shortcuts in the application:" -ForegroundColor Cyan
Write-Host "  Ctrl+Q or Ctrl+C   # Quit"
Write-Host "  Ctrl+T              # New tab"
Write-Host "  Ctrl+W              # Close tab"
Write-Host "  Ctrl+,              # Settings panel"
Write-Host "  Tab/Shift+Tab       # Switch tabs"
Write-Host "  Enter               # Send message"
