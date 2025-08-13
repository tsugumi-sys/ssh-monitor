#!/usr/bin/env bash
set -e

# Determine platform and architecture
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)
    FILE="ssh-monitor-aarch64-apple-darwin.tar.gz"
    ;;
  Darwin-x86_64)
    FILE="ssh-monitor-x86_64-apple-darwin.tar.gz"
    ;;
  Linux-x86_64)
    FILE="ssh-monitor-x86_64-unknown-linux-gnu.tar.gz"
    ;;
  *)
    echo "Unsupported OS or architecture: $(uname -s)-$(uname -m)"
    exit 1
    ;;
esac

echo "Downloading: $FILE ..."
curl -L -O "https://github.com/tsugumi-sys/ssh-monitor/releases/latest/download/$FILE"

echo "Extracting..."
tar -xzf "$FILE"

echo "Installing to /usr/local/bin/ ..."
sudo mv ssh-monitor /usr/local/bin/

echo "Cleaning up..."
rm "$FILE"

echo "✅ Installation complete! Run 'ssh-monitor --help' to see usage."