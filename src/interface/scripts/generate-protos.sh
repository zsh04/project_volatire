#!/bin/bash
# Generate TypeScript gRPC-web stubs from reflex.proto

set -e

PROTO_DIR="../../protos"
OUT_DIR="./lib/grpc/generated"

# Create output directory
mkdir -p "$OUT_DIR"

echo "Generating TypeScript gRPC-web stubs..."

# Generate using protoc with grpc-web plugin
protoc \
  --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
  --plugin=protoc-gen-grpc-web=./node_modules/.bin/protoc-gen-grpc-web \
  --js_out=import_style=commonjs,binary:"$OUT_DIR" \
  --ts_out=service=grpc-web:"$OUT_DIR" \
  --grpc-web_out=import_style=typescript,mode=grpcwebtext:"$OUT_DIR" \
  --proto_path="$PROTO_DIR" \
  "$PROTO_DIR/reflex.proto"

echo "✅ TypeScript stubs generated in $OUT_DIR"
