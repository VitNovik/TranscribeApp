#!/bin/bash

# Setup script for TranscribeApp development environment
# This script installs all required dependencies on macOS

set -e

echo "=========================================="
echo "TranscribeApp Development Setup"
echo "=========================================="

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Warning: This script is designed for macOS"
    echo "Some dependencies may need to be installed manually on other systems"
fi

# Check for Homebrew
if ! command -v brew &> /dev/null; then
    echo "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

echo ""
echo "Installing system dependencies..."

# Install FFmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo "Installing FFmpeg..."
    brew install ffmpeg
else
    echo "FFmpeg already installed"
fi

# Install Rust
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust already installed: $(rustc --version)"
fi

# Install Node.js
if ! command -v node &> /dev/null; then
    echo "Installing Node.js..."
    brew install node
else
    echo "Node.js already installed: $(node --version)"
fi

# Install Python (for whisper and pyannote)
if ! command -v python3 &> /dev/null; then
    echo "Installing Python..."
    brew install python
else
    echo "Python already installed: $(python3 --version)"
fi

echo ""
echo "Installing Python packages for transcription and diarization..."

# Install Python packages
pip3 install --upgrade pip
pip3 install openai-whisper
pip3 install pyannote.audio

echo ""
echo "Installing Node.js dependencies..."

# Install npm dependencies
npm install

echo ""
echo "Downloading default Whisper model..."

# Download default model
bash scripts/download_models.sh medium

echo ""
echo "=========================================="
echo "Setup complete!"
echo "=========================================="
echo ""
echo "To start development:"
echo "  npm run tauri:dev"
echo ""
echo "To build the application:"
echo "  npm run tauri:build"
echo ""
echo "Note: For pyannote.audio speaker diarization, you need a Hugging Face token."
echo "Set it with: export HF_TOKEN=your_token"
echo ""
