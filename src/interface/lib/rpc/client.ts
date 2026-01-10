import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import path from 'path';

const PROTO_PATH = path.join(process.cwd(), 'src/interface/lib/rpc/protos/reflex.proto');

// In production (Vercel/Docker), the path might be different.
// We rely on the build process to place the proto file in the correct location or use a relative path.
// For Next.js, process.cwd() is usually the project root.

const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true,
});

const reflexProto = grpc.loadPackageDefinition(packageDefinition).reflex as any;

let clientInstance: any = null;

export function getReflexClient() {
    if (clientInstance) {
        return clientInstance;
    }

    const grpcUrl = process.env.REFLEX_GRPC_URL || 'localhost:50051';

    // Create insecure credentials for now (internal network)
    // TODO: Use SSL/TLS if needed
    clientInstance = new reflexProto.ReflexService(
        grpcUrl,
        grpc.credentials.createInsecure()
    );

    return clientInstance;
}
