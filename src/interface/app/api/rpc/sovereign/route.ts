import { NextResponse } from 'next/server';
import type { SovereignCommandRequest, SovereignCommandResponse } from '@/lib/governance';
import { getReflexClient } from '@/lib/rpc/client';
import { secureCompare } from '@/lib/crypto';
import * as grpc from '@grpc/grpc-js';
import { cookies } from 'next/headers';

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

        // Directive-103: Sovereign Access Control
        // 1. Check for Environment Bypass
        const bypass = process.env.NEXT_PUBLIC_SOVEREIGN_BYPASS === 'true';

        // 2. Check for Sovereign Master Key Header
        const sovereignKey = request.headers.get('x-sovereign-key');

        if (!bypass) {
            // Secure Comparison against stored Master Key
            const validKey = process.env.NEXT_PUBLIC_SOVEREIGN_MASTER_KEY;

            // If bypass is OFF, we MUST have a valid Sovereign Key
            if (!sovereignKey || !validKey || !secureCompare(sovereignKey, validKey)) {
                return NextResponse.json(
                    { success: false, error: 'Invalid or missing Sovereign Key' },
                    { status: 401 }
                );
            }
        }

        // Compliance: Critical commands usually require biometric signature
        // If bypass is active, client sends a dummy signature
        const criticalCommands = ['KILL', 'CLOSE_ALL'];
        if (criticalCommands.includes(req.command) && !req.signature) {
            return NextResponse.json(
                { success: false, error: 'Biometric signature required' },
                { status: 403 }
            );
        }

        // Verify WebAuthn signature if present
        if (req.signature) {
            // If bypass is active, we skip strict crypto verification of the dummy signature
            if (!bypass) {
                const valid = await verifyWebAuthnSignature(req.signature, req.command);
                if (!valid) {
                    return NextResponse.json(
                        { success: false, error: 'Invalid biometric signature' },
                        { status: 403 }
                    );
                }
            }
        }

        // Special Case: VERIFY command
        // If we reached this point, the key is valid.
        if (req.command === 'VERIFY') {
            const latency_ms = Date.now() - startTime;
            return NextResponse.json({
                success: true,
                latency_ms,
            });
        }

        // Forward to Reflex via gRPC
        console.log(
            `SOVEREIGN COMMAND: ${req.command}`,
            req.payload ? `(payload: ${req.payload})` : ''
        );

        const client = getReflexClient();
        const metadata = new grpc.Metadata();
        if (sovereignKey) {
            metadata.add('x-sovereign-key', sovereignKey);
        }

        // Execute gRPC call based on command
        // Mapping:
        // KILL -> TriggerRatchet(KILL)
        // VETO -> TriggerVeto
        // PAUSE -> TriggerRatchet(FREEZE)
        // RESUME -> TriggerRatchet(IDLE)
        // CLOSE_ALL -> TriggerRatchet(TIGHTEN)
        // SET_SENTIMENT_OVERRIDE -> UpdateConfig(sentiment_override, payload)
        // CLEAR_SENTIMENT_OVERRIDE -> UpdateConfig(sentiment_override, -1)

        await new Promise<void>((resolve, reject) => {
            const callback = (err: any, response: any) => {
                if (err) return reject(err);
                if (response && !response.success) return reject(new Error(response.message || 'Unknown error'));
                resolve();
            };

            switch (req.command) {
                case 'KILL':
                    client.triggerRatchet({ level: 3, reason: 'Sovereign Command: KILL' }, metadata, callback);
                    break;
                case 'VETO':
                    client.triggerVeto({ reason: 'Sovereign Command: VETO', operator: 'Pilot' }, metadata, callback);
                    break;
                case 'PAUSE':
                    client.triggerRatchet({ level: 2, reason: 'Sovereign Command: PAUSE' }, metadata, callback);
                    break;
                case 'RESUME':
                    client.triggerRatchet({ level: 0, reason: 'Sovereign Command: RESUME' }, metadata, callback);
                    break;
                case 'CLOSE_ALL':
                    client.triggerRatchet({ level: 1, reason: 'Sovereign Command: CLOSE_ALL' }, metadata, callback);
                    break;
                case 'SET_SENTIMENT_OVERRIDE':
                    if (req.payload === undefined) return reject(new Error('Payload required for SET_SENTIMENT_OVERRIDE'));
                    client.updateConfig({ key: 'sentiment_override', value: req.payload }, metadata, callback);
                    break;
                case 'CLEAR_SENTIMENT_OVERRIDE':
                    client.updateConfig({ key: 'sentiment_override', value: -1 }, metadata, callback);
                    break;
                default:
                    reject(new Error(`Unknown command: ${req.command}`));
            }
        });

        // Determine User ID
        // Priority: X-User-ID header > Cookie > Default 'pilot'
        let userId = request.headers.get('x-user-id');
        if (!userId) {
            const cookieStore = cookies();
            const userIdCookie = cookieStore.get('sovereign_user_id');
            if (userIdCookie) {
                userId = userIdCookie.value;
            }
        }

        // Sanitize User ID (simple alpha-numeric check, fallback to 'pilot')
        const safeUserId = userId ? userId.replace(/[^a-zA-Z0-9_-]/g, '') : 'pilot';

        // Log to audit trail (QuestDB)
        await logSovereignCommand(req, safeUserId);

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
async function logSovereignCommand(req: SovereignCommandRequest, userId: string): Promise<void> {
    const logEntry = {
        timestamp: new Date().toISOString(),
        command: req.command,
        payload: req.payload ?? null,
        user_id: userId,
        signature: req.signature ? 'present' : 'none',
        latency_us: (Date.now() - req.timestamp) * 1000,
        source: 'WEB',
    };

    // Log structured JSON for log scraper to pick up and insert into QuestDB
    // This is a robust fallback if direct DB connection is flaky
    console.log(`QUESTDB_INSERT:sovereign_commands:${JSON.stringify(logEntry)}`);
}
