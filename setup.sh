#!/bin/bash

# Check if protoc is installed
if ! command -v protoc &> /dev/null; then
    echo "protoc not found. Installing..."

    # Install protoc (example for macOS using Homebrew)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install protobuf
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        sudo apt update
        sudo apt install -y protobuf-compiler
    else
        echo "Unsupported OS. Please install protoc manually."
        exit 1
    fi
fi

echo "protoc is installed."