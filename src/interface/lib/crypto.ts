import { timingSafeEqual } from 'crypto';

/**
 * Performs a constant-time comparison of two strings to prevent timing attacks.
 * Useful for validating API keys, tokens, or other secrets.
 */
export function secureCompare(a: string, b: string): boolean {
    const bufferA = Buffer.from(a);
    const bufferB = Buffer.from(b);

    if (bufferA.length !== bufferB.length) {
        return false;
    }

    return timingSafeEqual(bufferA, bufferB);
}
