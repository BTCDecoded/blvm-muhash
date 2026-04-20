#!/bin/bash
set -e
FUZZ_DIR="$(cd "$(dirname "$0")" && pwd)"
CORPUS_DIR="${1:-$FUZZ_DIR/corpus}"
mkdir -p "$CORPUS_DIR/muhash_num3072"
mkdir -p "$CORPUS_DIR/muhash_roll"
echo "Corpus dirs under $CORPUS_DIR"
