#!/usr/bin/env bash
# Install Ollama as its own system service, then download the triage model.
#
#   install-ollama.sh MODEL
#
# Ollama's official installer creates the `ollama` user and ollama.service
# (listening on 127.0.0.1:11434 only) and sets up AMD ROCm or NVIDIA CUDA
# when it finds the GPU. An Ollama that is already installed is left as is.
set -euo pipefail
model=${1:?usage: install-ollama.sh MODEL}

if ! command -v ollama > /dev/null; then
  [ "${ALLOW_DEPENDENCY_INSTALL:-}" = 1 ] || {
    echo 'Ollama is missing. Install it first or explicitly allow installation with ALLOW_DEPENDENCY_INSTALL=1.' >&2
    exit 1
  }
  echo "installing Ollama from ollama.com"
  installer=$(mktemp)
  trap 'rm -f "$installer"' EXIT
  curl -fsSL https://ollama.com/install.sh -o "$installer"
  sh "$installer"
fi
systemctl enable --now --quiet ollama.service

for _ in $(seq 30); do
  ollama list > /dev/null 2>&1 && break
  sleep 1
done
echo "downloading model $model (first time only)"
ollama pull "$model"
