#!/bin/bash

# Download Whisper models for TranscribeApp
# Models are downloaded from Hugging Face

set -e

MODELS_DIR="$HOME/.transcribe/models"
mkdir -p "$MODELS_DIR"

BASE_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main"

# Available models
declare -A MODELS=(
    ["tiny"]="ggml-tiny.bin"
    ["base"]="ggml-base.bin"
    ["small"]="ggml-small.bin"
    ["medium"]="ggml-medium.bin"
    ["large-v3"]="ggml-large-v3.bin"
)

# Default model to download
DEFAULT_MODEL="medium"

download_model() {
    local model_name=$1
    local model_file=${MODELS[$model_name]}

    if [ -z "$model_file" ]; then
        echo "Unknown model: $model_name"
        echo "Available models: ${!MODELS[@]}"
        exit 1
    fi

    local target_path="$MODELS_DIR/$model_file"

    if [ -f "$target_path" ]; then
        echo "Model $model_name already exists at $target_path"
        return 0
    fi

    echo "Downloading $model_name model..."
    curl -L --progress-bar -o "$target_path" "$BASE_URL/$model_file"

    if [ $? -eq 0 ]; then
        echo "Successfully downloaded $model_name to $target_path"
    else
        echo "Failed to download $model_name"
        rm -f "$target_path"
        exit 1
    fi
}

# Parse arguments
if [ $# -eq 0 ]; then
    echo "No model specified, downloading default model: $DEFAULT_MODEL"
    download_model "$DEFAULT_MODEL"
elif [ "$1" == "all" ]; then
    echo "Downloading all models..."
    for model in "${!MODELS[@]}"; do
        download_model "$model"
    done
elif [ "$1" == "list" ]; then
    echo "Available models:"
    for model in "${!MODELS[@]}"; do
        file=${MODELS[$model]}
        if [ -f "$MODELS_DIR/$file" ]; then
            echo "  $model (installed)"
        else
            echo "  $model"
        fi
    done
else
    download_model "$1"
fi

echo ""
echo "Models directory: $MODELS_DIR"
echo "Done!"
