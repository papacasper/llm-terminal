# LLM Terminal

A modern, Warp-inspired **cross-platform** desktop terminal emulator for chatting with multiple LLM providers (Claude and OpenAI). Built with Rust and featuring tabs, block-based conversations, and a professional GUI interface that works seamlessly across Windows, macOS, and Linux.

## 🎯 **CONVERSION COMPLETE: TUI → GUI Desktop App**

This application has been successfully converted from a **Terminal UI (TUI)** application that ran inside existing terminals to a **standalone desktop GUI application** with its own built-in terminal emulator.

### What Changed:
- **Before**: Used `ratatui` + `crossterm` for terminal-based UI
- **After**: Uses `egui` + `eframe` for native desktop GUI
- **Interface**: Now opens as a desktop window instead of running in terminal
- **Features**: Maintains all original functionality in a modern GUI

## 🌟 Features

- **🖥️ Cross-Platform**: Works on Windows, macOS, and Linux
- **🐚 Universal Terminal Support**: Compatible with cmd, PowerShell, bash, zsh, fish, and more
- **📑 Multi-tab Interface**: Create and manage multiple conversation tabs
- **🤖 Multiple LLM Providers**: Support for both Anthropic Claude and OpenAI GPT-4
- **👥 Multi-Agent Tasks**: Run several agents in parallel for complex workflows
- **📂 Context Loader**: Provide codebase context to AI agents automatically
- **🔁 Scriptable Workflows**: Define reusable sets of terminal commands
- **🔒 Telemetry Toggle**: Control optional usage reporting
- **🎨 Modern Terminal UI**: Clean, professional interface using ratatui
- **⚡ Real-time Async Communication**: Non-blocking API calls with response handling
- **⌨️ Keyboard Shortcuts**: Power-user friendly navigation and controls
- **⚙️ Settings Panel**: Easy API key management and configuration
- **💬 Block-based Conversations**: Warp-style conversation blocks for better readability
- **🔧 Easy Setup**: Interactive setup scripts for all platforms

## 📦 Installation

### Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
   ```bash
   # This works on all platforms
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Git**: For cloning the repository

### Quick Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/llm-terminal.git
   cd llm-terminal
   ```

2. **Choose your setup method**:

   **Option A: Interactive Setup (Recommended)**
   ```bash
   # On Windows (Command Prompt)
   setup_env.bat
   
   # On Windows (PowerShell) or cross-platform PowerShell
   ./setup_env.ps1
   
   # On macOS/Linux (bash/zsh)
   source setup_env.sh
   
   # Using Make (if available)
   make setup
   ```

   **Option B: Manual Build**
   ```bash
   # Build the project
   cargo build --release
   
   # Run directly
   cargo run
   ```

## 🔑 API Key Setup

The application supports multiple ways to set up API keys across different platforms and terminals:

### Environment Variables (Recommended)

**Windows Command Prompt:**
```cmd
set ANTHROPIC_API_KEY=your-claude-api-key
set OPENAI_API_KEY=your-openai-api-key
```

**Windows PowerShell:**
```powershell
$env:ANTHROPIC_API_KEY = "your-claude-api-key"
$env:OPENAI_API_KEY = "your-openai-api-key"
```

**macOS/Linux (bash/zsh):**
```bash
export ANTHROPIC_API_KEY="your-claude-api-key"
export OPENAI_API_KEY="your-openai-api-key"
```

**Fish Shell:**
```fish
set -x ANTHROPIC_API_KEY "your-claude-api-key"
set -x OPENAI_API_KEY "your-openai-api-key"
```

### API Usage

`llm-terminal` communicates directly with the official APIs from Anthropic and OpenAI:

- **Claude API**: `https://api.anthropic.com/v1/messages`
- **OpenAI API**: `https://api.openai.com/v1/chat/completions`

Make sure your API keys have access to these endpoints. The selected model name is passed in each request, so you can use any model your key has permission for.

# Optional telemetry
export LLM_TERMINAL_TELEMETRY=false
### Configuration File

Alternatively, create a configuration file at:
- **Windows**: `%APPDATA%\llm-terminal\config.toml`
- **macOS**: `~/Library/Application Support/llm-terminal/config.toml`
- **Linux**: `~/.config/llm-terminal/config.toml`

```toml
[settings]
default_provider = "Claude"

# API keys (optional - environment variables take precedence)
claude_api_key = "your-claude-api-key"
openai_api_key = "your-openai-api-key"
```

## 🚀 Running the Application

### Quick Start Commands

**Using the built executable:**
```bash
# Windows
target\release\llm-terminal.exe

# macOS/Linux
./target/release/llm-terminal
```

**Using Cargo:**
```bash
cargo run
```

**Using Make (if available):**
```bash
make run        # Build and run
make run-dev    # Run from source
```

### Platform-Specific Notes

**Windows:**
- Works in Command Prompt, PowerShell, Windows Terminal
- Supports both PowerShell 5.1 and PowerShell Core 7+
- Terminal colors and Unicode support depend on your terminal

**macOS:**
- Works in Terminal.app, iTerm2, and other terminal emulators
- Supports both Intel and Apple Silicon Macs
- Full Unicode and color support

**Linux:**
- Works in GNOME Terminal, Konsole, Alacritty, and most terminal emulators
- Supports various distributions (Ubuntu, Fedora, Arch, etc.)
- Full Unicode and color support

## Usage

### Keyboard Shortcuts

- **Ctrl+Q** or **Ctrl+C**: Quit the application
- **Ctrl+T**: Create a new tab
- **Ctrl+W**: Close the current tab
- **Ctrl+,**: Toggle settings panel
- **Tab**: Switch to next tab
- **Shift+Tab**: Switch to previous tab
- **Enter**: Send message
- **Esc**: Return to chat mode (from settings)

### Interface Layout

```
┌─────────────────────────────────────────────────────┐
│ Tab1 | Tab2 | Tab3 | ...                            │ ← Tab bar
├─────────────────────────────────────────────────────┤
│                                                     │
│    Chat Messages (Block-based)                      │ ← Main content
│    You: Hello                                       │
│    Claude: Hi there! How can I help you today?     │
│                                                     │
├─────────────────────────────────────────────────────┤
│ Type your message here...                           │ ← Input area
├─────────────────────────────────────────────────────┤
│ Ready | Provider: Claude | Tab: 1 | Ctrl+, settings│ ← Status bar
└─────────────────────────────────────────────────────┘
```

### Features

#### Multiple Tabs
- Each tab maintains its own conversation history
- Tabs can use different LLM providers
- Visual indicators show which tab is waiting for a response

#### Provider Support
- **Claude**: Uses Anthropic's latest claude-3-5-sonnet model
- **OpenAI**: Uses GPT-4o model
- Automatic provider detection based on available API keys

#### Settings Panel
- Press **Ctrl+,** to open the settings panel
- Shows API key configuration status
- Displays available providers and current settings
- Includes keyboard shortcut reference

## Project Structure

```
llm-terminal/
├── Cargo.toml              # Dependencies and project config
├── src/
│   ├── main.rs             # Main entry point and UI rendering
│   ├── app.rs              # Application state and event handling
│   ├── config.rs           # Configuration management
│   ├── models.rs           # Data structures and types
│   ├── llm/                # LLM client implementations
│   │   ├── mod.rs          # Module exports
│   │   ├── client.rs       # HTTP client wrapper
│   │   ├── claude.rs       # Anthropic Claude API
│   │   └── openai.rs       # OpenAI API
│   └── ui/                 # UI components
│       ├── mod.rs          # UI module exports
│       ├── chat.rs         # Chat interface rendering
│       ├── settings.rs     # Settings panel
│       └── input.rs        # Input handling
└── README.md               # This file
```

## Configuration

The application loads settings from:
1. Environment variables (highest priority)
2. Configuration file (fallback)

Configuration file location:
- **Windows**: `%APPDATA%/llm-terminal/config.toml`
- **macOS**: `~/Library/Application Support/llm-terminal/config.toml`
- **Linux**: `~/.config/llm-terminal/config.toml`

## Development

### Running Tests
```bash
cargo test
```

### Building for Release
```bash
cargo build --release
```

### Adding New Features

The application is designed for extensibility:

1. **New LLM Providers**: Implement the `LLMClient` trait in `src/llm/`
2. **UI Components**: Add new rendering functions in `src/ui/`
3. **Settings**: Extend the `Settings` struct in `src/models.rs`

## Dependencies

- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal manipulation
- **tokio**: Async runtime
- **reqwest**: HTTP client for API calls
- **serde**: Serialization/deserialization
- **anyhow**: Error handling
- **uuid**: Unique identifier generation
- **chrono**: Date/time handling

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Troubleshooting

### Common Issues

1. **"No client available for provider"**: Make sure you have the correct API key environment variable set
2. **API errors**: Check that your API keys are valid and have sufficient credits
3. **Terminal display issues**: Ensure your terminal supports UTF-8 and has sufficient size

### API Key Setup

Make sure your API keys are correctly set:
```bash
# Check if environment variables are set
echo $ANTHROPIC_API_KEY
echo $OPENAI_API_KEY
```

### Terminal Compatibility

**Supported Shells & Terminals:**
- **Windows**: cmd, PowerShell 5.1/7+, Windows Terminal, ConEmu
- **macOS**: bash, zsh, fish, Terminal.app, iTerm2, Hyper
- **Linux**: bash, zsh, fish, GNOME Terminal, Konsole, Alacritty, Kitty
- **Cross-platform**: Any terminal that supports ANSI escape codes

**Requirements:**
- Terminal size: Minimum 80x24 characters
- Unicode support (UTF-8) for proper emoji and character display
- ANSI color support for syntax highlighting

**Tested Environments:**
- Windows 10/11 (Command Prompt, PowerShell, Windows Terminal)
- macOS 12+ (Terminal.app, iTerm2)
- Ubuntu 20.04+ (GNOME Terminal)
- Fedora 35+ (GNOME Terminal, Konsole)
- Arch Linux (Alacritty, Kitty)

### Performance

- The application uses async I/O for non-blocking API calls
- UI updates at 250ms intervals for smooth responsiveness
- Messages are cached locally for each tab
- Cross-platform binary with minimal dependencies
- Memory efficient with lazy loading of chat history
