#!/bin/zsh
# scripts/compile_protos.zsh
set -e # Exit immediately on error

PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd "$PROJECT_ROOT"

echo "🧠 Compiling Nervous System for Python..."

# 1. Define Paths
VENV_BIN="src/brain/.venv/bin"
OUT_DIR="src/brain/app/generated"
PLUGIN_PATH="$VENV_BIN/protoc-gen-mypy"

# 2. Check for Plugin existence
if [ ! -f "$PLUGIN_PATH" ]; then
    echo "❌ Error: protoc-gen-mypy not found at $PLUGIN_PATH"
    echo "   -> Run: uv pip install --python src/brain/.venv mypy-protobuf"
    exit 1
fi

# 3. Create output structure
mkdir -p "$OUT_DIR"
touch "$OUT_DIR/__init__.py"

# 4. Compile with Explicit Plugin Path
# We use the python inside the venv to run grpc_tools
"$VENV_BIN/python" -m grpc_tools.protoc \
    -I protos \
    --plugin=protoc-gen-mypy="$PLUGIN_PATH" \
    --python_out="$OUT_DIR" \
    --grpc_python_out="$OUT_DIR" \
    --mypy_out="$OUT_DIR" \
    protos/brain.proto protos/reflex.proto

# 5. Fix relative imports (The standard gRPC hack)
# gRPC generates 'import brain_pb2', but inside a package it needs 'from . import brain_pb2'
sed -i '' 's/^import brain_pb2/from . import brain_pb2/' "$OUT_DIR/brain_pb2_grpc.py"
sed -i '' 's/^import reflex_pb2/from . import reflex_pb2/' "$OUT_DIR/reflex_pb2_grpc.py"

echo "✅ Python Synapses Built."
ls -1 "$OUT_DIR"