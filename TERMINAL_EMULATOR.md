# Built-in Terminal Emulator

Your LLM Terminal now includes a built-in terminal emulator that runs directly within the application! No need to switch to external terminal windows.

## 🚀 Features

- **Embedded Terminal**: Run shell commands directly in the app
- **Multiple Terminal Sessions**: Create and manage multiple terminal tabs
- **Cross-Platform**: Works on Windows (PowerShell/cmd), macOS, and Linux
- **Real-time Output**: See command output as it happens
- **Command History**: Keep track of executed commands with timestamps
- **Session Management**: Create, switch, and close terminal sessions

## 📋 Usage

### Mode Switching
- **Ctrl+,**: Cycle through modes (Chat → Terminal → Settings → Chat)
- The terminal mode gives you a fully functional embedded terminal

### Terminal Controls
- **Ctrl+Shift+T**: Create new terminal session
- **Ctrl+W**: Close current terminal session (if more than one exists)
- **Tab**: Switch to next terminal session
- **Shift+Tab**: Switch to previous terminal session
- **Enter**: Execute the current command
- **Backspace**: Delete characters from input
- **Type normally**: Enter commands as you would in any terminal

### Example Commands (Windows/PowerShell)
```powershell
# List directory contents
ls
dir

# Change directory
cd C:\Users

# Run Python scripts
python script.py

# Check system info
systeminfo

# Network info
ipconfig
```

### Example Commands (Linux/macOS)
```bash
# List directory contents
ls -la

# Change directory
cd /home/user

# System info
uname -a

# Network info
ifconfig

# Process info
ps aux
```

## 🎯 Use Cases

1. **Quick Commands**: Run one-off commands without leaving the chat interface
2. **Development Work**: Execute build scripts, run tests, check git status
3. **System Administration**: Monitor processes, check system status
4. **File Management**: Navigate directories, copy files, manage permissions
5. **Multi-tasking**: Keep multiple terminal sessions for different projects

## 🔧 Technical Details

- **Shell Detection**: Automatically detects and uses the best available shell:
  - Windows: PowerShell Core (pwsh) → PowerShell 5.1 → cmd
  - macOS/Linux: Uses `$SHELL` environment variable or defaults to bash
- **Process Management**: Each terminal session runs in its own isolated process
- **Output Buffering**: Real-time output capture with proper error handling
- **Session Isolation**: Each tab maintains its own working directory and command history

## 💡 Tips

- Use **Ctrl+Shift+T** to create specialized terminal sessions for different tasks
- Terminal sessions persist their working directory and history until closed
- Long-running commands will continue to execute and show output in real-time
- Error output is clearly marked and highlighted in red
- System messages (like session start) are shown in yellow

## 🚨 Important Notes

- Terminal sessions are isolated - changing directory in one doesn't affect others
- Processes are automatically terminated when sessions are closed
- Interactive programs (like text editors) may not work perfectly in the embedded terminal
- For complex terminal work, you can still use external terminals alongside the embedded one

## 🎮 Keyboard Shortcuts Summary

| Shortcut | Action |
|----------|--------|
| `Ctrl+,` | Cycle modes (Chat/Terminal/Settings) |
| `Ctrl+Shift+T` | New terminal session |
| `Ctrl+W` | Close terminal session |
| `Tab` | Next terminal session |
| `Shift+Tab` | Previous terminal session |
| `Enter` | Execute command |
| `Backspace` | Delete character |
| `Ctrl+Q` or `Ctrl+C` | Quit application |

Enjoy your new embedded terminal experience! 🎉
