@echo off
REM Cross-platform batch script for Windows Command Prompt
REM LLM Terminal Environment Setup
REM Usage: setup_env.bat

echo.
echo 🚀 LLM Terminal Environment Setup
echo ==================================

REM Check if Cargo is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Cargo not found. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo.
echo 🔍 Checking existing environment variables...

REM Check existing environment variables
if defined ANTHROPIC_API_KEY (
    echo ✅ ANTHROPIC_API_KEY is set
    set "CLAUDE_CONFIGURED=1"
) else if defined CLAUDE_API_KEY (
    echo ✅ CLAUDE_API_KEY is set
    set "ANTHROPIC_API_KEY=%CLAUDE_API_KEY%"
    set "CLAUDE_CONFIGURED=1"
) else (
    echo ⚠️  No Claude API key found
    set "CLAUDE_CONFIGURED=0"
)

if defined OPENAI_API_KEY (
    echo ✅ OPENAI_API_KEY is set
    set "OPENAI_CONFIGURED=1"
) else (
    echo ⚠️  No OpenAI API key found
    set "OPENAI_CONFIGURED=0"
)

REM Interactive setup if no keys are found
if %CLAUDE_CONFIGURED%==0 if %OPENAI_CONFIGURED%==0 (
    echo.
    echo 🔑 No API keys found. Let's set them up interactively.
    echo You can press Enter to skip any provider you don't want to use.
    echo.
    
    echo 📝 Setting up Anthropic Claude API Key
    set /p "CLAUDE_INPUT=Enter your Anthropic Claude API key (or press Enter to skip): "
    if not "!CLAUDE_INPUT!"=="" (
        set "ANTHROPIC_API_KEY=!CLAUDE_INPUT!"
        echo ✅ Claude API key set successfully
        set "CLAUDE_CONFIGURED=1"
    ) else (
        echo ⚠️  Claude API key not set - this provider will be unavailable
    )
    
    echo.
    echo 📝 Setting up OpenAI API Key
    set /p "OPENAI_INPUT=Enter your OpenAI API key (or press Enter to skip): "
    if not "!OPENAI_INPUT!"=="" (
        set "OPENAI_API_KEY=!OPENAI_INPUT!"
        echo ✅ OpenAI API key set successfully
        set "OPENAI_CONFIGURED=1"
    ) else (
        echo ⚠️  OpenAI API key not set - this provider will be unavailable
    )
)

REM Build the application
echo.
echo 🔨 Building LLM Terminal...
cargo build --release
if %errorlevel% neq 0 (
    echo ❌ Build failed!
    pause
    exit /b 1
)
echo ✅ Build successful!

REM Set executable path
set "EXECUTABLE_PATH=target\release\llm-terminal.exe"

REM Display setup summary
echo.
echo 📋 Setup Summary
echo ================
echo Platform: Windows
echo Executable: %EXECUTABLE_PATH%
if %CLAUDE_CONFIGURED%==1 (
    echo Anthropic Claude: ✅ Configured
) else (
    echo Anthropic Claude: ❌ Not configured
)
if %OPENAI_CONFIGURED%==1 (
    echo OpenAI: ✅ Configured
) else (
    echo OpenAI: ❌ Not configured
)

echo.
echo 🎉 Setup complete!
echo.
echo 💡 Quick start commands:
echo   %EXECUTABLE_PATH%  # Start the application
echo   cargo run                    # Run from source
echo   cargo test                   # Run tests
echo.
echo 🔧 Keyboard shortcuts in the application:
echo   Ctrl+Q or Ctrl+C   # Quit
echo   Ctrl+T              # New tab
echo   Ctrl+W              # Close tab
echo   Ctrl+,              # Settings panel
echo   Tab/Shift+Tab       # Switch tabs
echo   Enter               # Send message
echo.

REM Ask if user wants to run the application now
set /p "RUN_NOW=Would you like to run LLM Terminal now? (y/n): "
if /i "%RUN_NOW%"=="y" (
    echo.
    echo 🚀 Starting LLM Terminal...
    echo Press Ctrl+Q to quit, Ctrl+, for settings
    echo.
    "%EXECUTABLE_PATH%"
)

pause
