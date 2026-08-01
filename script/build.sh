cd "$(dirname "${BASH_SOURCE[0]}")"
cd ../lora

export GIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

cargo build $@