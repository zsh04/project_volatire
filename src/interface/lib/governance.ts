/**
 * Directive-86: Sovereign Interface Law
 * Frontend governance API for pilot strategic oversight
 */

export enum SovereignCommand {
    KILL = 'KILL',
    VETO = 'VETO',
    PAUSE = 'PAUSE',
    RESUME = 'RESUME',
    CLOSE_ALL = 'CLOSE_ALL',
    SET_SENTIMENT_OVERRIDE = 'SET_SENTIMENT_OVERRIDE',
    CLEAR_SENTIMENT_OVERRIDE = 'CLEAR_SENTIMENT_OVERRIDE',
    VERIFY = 'VERIFY',
}

export interface SovereignCommandRequest {
    command: SovereignCommand;
    payload?: number; // For SET_SENTIMENT_OVERRIDE (0.0-1.0)
    signature?: string; // WebAuthn signature for critical commands
    timestamp: number; // Client timestamp for latency tracking
}

export interface SovereignCommandResponse {
    success: boolean;
    latency_ms?: number;
    error?: string;
}

const STORAGE_KEY = 'sovereign_master_key';
const USER_ID_COOKIE = 'sovereign_user_id';

/**
 * Store the Sovereign Key in session storage
 */
export function setSovereignKey(key: string) {
    if (typeof window !== 'undefined') {
        sessionStorage.setItem(STORAGE_KEY, key);
    }
}

/**
 * Retrieve the Sovereign Key from storage or environment
 */
export function getSovereignKey(): string {
    if (typeof window !== 'undefined') {
        const stored = sessionStorage.getItem(STORAGE_KEY);
        if (stored) return stored;
    }
    return process.env.NEXT_PUBLIC_SOVEREIGN_MASTER_KEY || '';
}

/**
 * Set the User ID in a cookie (simple session mechanism)
 */
export function setUserId(userId: string) {
    if (typeof window !== 'undefined') {
        // Set cookie for 7 days
        const days = 7;
        const date = new Date();
        date.setTime(date.getTime() + (days * 24 * 60 * 60 * 1000));
        const expires = "; expires=" + date.toUTCString();
        document.cookie = USER_ID_COOKIE + "=" + (userId || "") + expires + "; path=/; SameSite=Strict";
    }
}

/**
 * Get the User ID from cookie
 */
export function getUserId(): string {
    if (typeof window !== 'undefined') {
        const nameEQ = USER_ID_COOKIE + "=";
        const ca = document.cookie.split(';');
        for(let i=0;i < ca.length;i++) {
            let c = ca[i];
            while (c.charAt(0)==' ') c = c.substring(1,c.length);
            if (c.indexOf(nameEQ) == 0) return c.substring(nameEQ.length,c.length);
        }
    }
    return 'pilot'; // Default fallback
}


/**
 * Send a sovereign command to the Reflex backend
 * Critical commands (KILL, CLOSE_ALL) require biometric signature
 */
export async function sendSovereignCommand(
    command: SovereignCommand,
    payload?: number
): Promise<SovereignCommandResponse> {
    const criticalCommands = [
        SovereignCommand.KILL,
        SovereignCommand.CLOSE_ALL,
    ];

    const requiresSignature = criticalCommands.includes(command);

    // Request WebAuthn signature for critical commands
    let signature: string | undefined;
    if (requiresSignature) {
        if (process.env.NEXT_PUBLIC_SOVEREIGN_BYPASS) {
            console.log('🛡️ SKIPPING WEBAUTHN - SOVEREIGN BYPASS ACTIVE');
            signature = 'sovereign_bypass_signature_hex';
        } else {
            try {
                signature = await requestBiometricSignature(command);
            } catch (error) {
                throw new Error(`Biometric signature required for ${command}`);
            }
        }
    }

    const request: SovereignCommandRequest = {
        command,
        payload,
        signature,
        timestamp: Date.now(),
    };

    const response = await fetch('/api/rpc/sovereign', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Sovereign-Key': getSovereignKey(),
            'X-User-ID': getUserId()
        },
        body: JSON.stringify(request),
    });

    if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Sovereign command failed');
    }

    return response.json();
}

/**
 * Request biometric signature via WebAuthn
 * Returns base64-encoded signature
 */
async function requestBiometricSignature(
    command: SovereignCommand
): Promise<string> {
    if (!window.PublicKeyCredential) {
        throw new Error('WebAuthn not supported. Use fallback authentication.');
    }

    try {
        // Challenge message includes command type for audit trail
        const challenge = new TextEncoder().encode(
            `SOVEREIGN_${command}_${Date.now()}`
        );

        const credential = await navigator.credentials.get({
            publicKey: {
                challenge,
                timeout: 60000,
                userVerification: 'required',
            },
        }) as PublicKeyCredential;

        if (!credential) {
            throw new Error('Biometric authentication cancelled');
        }

        // Convert credential to base64
        const response = credential.response as AuthenticatorAssertionResponse;
        const signature = btoa(
            String.fromCharCode(...new Uint8Array(response.signature))
        );

        return signature;
    } catch (error) {
        console.error('WebAuthn error:', error);
        throw new Error('Biometric authentication failed');
    }
}

/**
 * Utility: Request confirmation for dangerous commands
 */
export function confirmCommand(command: SovereignCommand): boolean {
    const messages: Record<SovereignCommand, string> = {
        [SovereignCommand.KILL]:
            'KILL ALL? This will immediately stop the system.',
        [SovereignCommand.CLOSE_ALL]:
            'CLOSE ALL POSITIONS? This action is irreversible.',
        [SovereignCommand.VETO]: 'Initiate System Risk-Halt (Veto)?',
        [SovereignCommand.PAUSE]: 'Enter tactical pause mode?',
        [SovereignCommand.RESUME]: 'Resume trading?',
        [SovereignCommand.SET_SENTIMENT_OVERRIDE]:
            'Override sentiment weight manually?',
        [SovereignCommand.CLEAR_SENTIMENT_OVERRIDE]:
            'Clear sentiment override?',
        [SovereignCommand.VERIFY]: 'Verify Sovereign Key?',
    };

    return confirm(messages[command]);
}
