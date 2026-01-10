import { secureCompare } from '../lib/crypto';

describe('secureCompare', () => {
    it('should return true for identical strings', () => {
        const secret = 'super-secret-key';
        expect(secureCompare(secret, secret)).toBe(true);
    });

    it('should return false for different strings', () => {
        const secret = 'super-secret-key';
        const wrong = 'wrong-key';
        expect(secureCompare(secret, wrong)).toBe(false);
    });

    it('should return false if lengths differ', () => {
        const secret = 'super-secret-key';
        const wrong = 'super-secret-ke';
        expect(secureCompare(secret, wrong)).toBe(false);
    });
});
