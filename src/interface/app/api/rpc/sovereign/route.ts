import { NextResponse } from 'next/server';
import type { SovereignCommandRequest, SovereignCommandResponse } from '@/lib/governance';

/**
 * Directive-86: Sovereign Command API Endpoint
 * Handles high-priority pilot intervention commands
 */
export async function POST(request: Request): Promise<NextResponse<SovereignCommandResponse>> {
    const startTime = Date.now();

    try {
        const req: SovereignCommandRequest = await request.json();

        // Validate required fields
        if (!req.command || !req.timestamp) {
            return NextResponse.json(
                { success: false, error: 'Missing required fields' },
                { status: 400 }
            );
        }

        // Critical commands require biometric signature
        const criticalCommands = ['KILL', 'CLOSE_ALL'];
        if (criticalCommands.includes(req.command) && !req.signature) {
            return NextResponse.json(
                { success: false, error: 'Biometric signature required' },
                { status: 403 }
            );
        }

        // TODO: Verify WebAuthn signature
        if (req.signature) {
            const valid = await verifyWebAuthnSignature(req.signature, req.command);
            if (!valid) {
                return NextResponse.json(
                    { success: false, error: 'Invalid biometric signature' },
                    { status: 403 }
                );
            }
        }

        // Forward to Reflex via gRPC
        const grpcUrl = process.env.REFLEX_GRPC_URL || 'http://localhost:50051';

        // TODO: Implement actual gRPC call to Reflex
        // For now, log the command
        console.log(
            `SOVEREIGN COMMAND: ${req.command}`,
            req.payload ? `(payload: ${req.payload})` : ''
        );

        // Log to audit trail (QuestDB)
        await logSovereignCommand(req);

        const latency_ms = Date.now() - startTime;

        return NextResponse.json({
            success: true,
            latency_ms,
        });
    } catch (error) {
        console.error('Sovereign command error:', error);
        return NextResponse.json(
            {
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error'
            },
            { status: 500 }
        );
    }
}

/**
 * Verify WebAuthn signature
 * TODO: Implement full WebAuthn verification
 */
async function verifyWebAuthnSignature(
    signature: string,
    command: string
): Promise<boolean> {
    // Placeholder implementation
    console.log(`Verifying signature for ${command}:`, signature.substring(0, 20) + '...');

    // In production, this would:
    // 1. Decode base64 signature
    // 2. Verify against stored public key
    // 3. Check challenge matches command
    // 4. Verify signature timestamp

    return true; // Accept for now (DEVELOPMENT ONLY)
}

/**
 * Log sovereign command to QuestDB audit trail
 */
async function logSovereignCommand(req: SovereignCommandRequest): Promise<void> {
    const logEntry = {
        timestamp: new Date().toISOString(),
        command: req.command,
        payload: req.payload ?? null,
        user_id: 'pilot', // TODO: Get from session
        signature: req.signature ? 'present' : 'none',
        latency_us: (Date.now() - req.timestamp) * 1000,
        source: 'WEB',
    };

    // Log structured JSON for log scraper to pick up and insert into QuestDB
    // This is a robust fallback if direct DB connection is flaky
    console.log(`QUESTDB_INSERT:sovereign_commands:${JSON.stringify(logEntry)}`);

    // In a full production environment with QuestDB client:
    /*
    const sender = new Sender();
    await sender.connect({ port: 9009 });
    await sender.table('sovereign_commands')
      .symbol('command', req.command)
      .symbol('user_id', 'pilot')
      .symbol('source', 'WEB')
      .string('signature', req.signature || '')
      .float('payload', req.payload || 0.0)
      .long('latency_us', logEntry.latency_us)
      .at(Date.now(), 'ms');
    await sender.flush();
    await sender.close();
    */
}
