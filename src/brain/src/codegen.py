import os
import sys
from grpc_tools import protoc


def generate_protos():
    # Paths
    # Workspace root assuming we are in src/brain/src likely or running from root?
    # Let's assume this script is run from src/brain via `python src/codegen.py` or similar.
    # Better: Use relative paths from this file.

    current_dir = os.path.dirname(os.path.abspath(__file__))  # src/brain/src
    project_root = os.path.abspath(os.path.join(current_dir, "../../../"))  # voltaire
    proto_dir = os.path.join(project_root, "protos")
    out_dir = os.path.join(current_dir, "generated")

    print(f"Generating protos from {proto_dir} to {out_dir}")

    if not os.path.exists(out_dir):
        os.makedirs(out_dir)

    protos = ["brain.proto", "reflex.proto"]

    for proto in protos:
        proto_path = os.path.join(proto_dir, proto)
        if not os.path.exists(proto_path):
            print(f"Error: {proto_path} not found")
            continue

        # Command: protoc -I=... --python_out=... --grpc_python_out=... ...
        command = [
            "grpc_tools.protoc",
            f"-I{proto_dir}",
            f"--python_out={out_dir}",
            f"--grpc_python_out={out_dir}",
            proto_path,
        ]

        exit_code = protoc.main(command)
        if exit_code != 0:
            print(f"Failed to generate {proto}")
            sys.exit(exit_code)

    # Fix imports in generated files (Module 'generated' vs absolute)
    # Generated files use "import brain_pb2" which expects brain_pb2 in path.
    # Since they are in `generated` package, we might need to conform.
    # But usually adding `src/brain/src` to PYTHONPATH is enough.
    print("Success.")


if __name__ == "__main__":
    generate_protos()
