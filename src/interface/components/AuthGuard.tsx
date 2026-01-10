import React, { useEffect, useState } from 'react';
import { SovereignCommand, sendSovereignCommand, setSovereignKey, getSovereignKey, setUserId, getUserId } from '../lib/governance';

const BYPASS_KEY = process.env.NEXT_PUBLIC_SOVEREIGN_BYPASS;

interface AuthGuardProps {
    children: React.ReactNode;
}

export function AuthGuard({ children }: AuthGuardProps) {
    const [isAuthenticated, setIsAuthenticated] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const [keyInput, setKeyInput] = useState('');
    const [userIdInput, setUserIdInput] = useState('');
    const [error, setError] = useState<string | null>(null);
    const [isVerifying, setIsVerifying] = useState(false);

    const verifyKey = async (key: string, userId: string = 'pilot') => {
        setIsVerifying(true);
        setError(null);
        try {
            // Optimistically set the key in storage so sendSovereignCommand can use it
            setSovereignKey(key);
            setUserId(userId);

            // Verify with the server
            await sendSovereignCommand(SovereignCommand.VERIFY);

            setIsAuthenticated(true);
        } catch (err) {
            console.error('Verification failed', err);
            setError('Invalid Sovereign Key');
            setSovereignKey(''); // Clear invalid key
            setIsAuthenticated(false);
        } finally {
            setIsVerifying(false);
            setIsLoading(false);
        }
    };

    useEffect(() => {
        // Check for Sovereign Bypass
        if (BYPASS_KEY) {
            console.log('🛡️ SOVEREIGN BYPASS ACTIVE');
            setIsAuthenticated(true);
            setIsLoading(false);
            return;
        }

        // Check if we already have a key in storage
        const storedKey = getSovereignKey();
        const storedUser = getUserId();

        if (storedKey) {
            setKeyInput(storedKey); // Pre-fill key if available? No, verification happens automatically but if fails we want input empty or filled?
            // Better to just try verifying
            // Note: verifyKey takes arguments, but here we might want to pass storedUser
            verifyKey(storedKey, storedUser);
        } else {
            setIsLoading(false);
        }
    }, []);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        verifyKey(keyInput, userIdInput || 'pilot');
    };

    if (isLoading) {
        return (
            <div className="flex h-screen items-center justify-center bg-black text-white font-mono">
                <div className="flex flex-col items-center gap-4">
                    <div className="w-6 h-6 border-2 border-emerald-500 border-t-transparent rounded-full animate-spin" />
                    <span className="text-xs tracking-widest text-emerald-500/50">AUTHENTICATING...</span>
                </div>
            </div>
        );
    }

    if (!isAuthenticated) {
        return (
            <div className="flex h-screen items-center justify-center bg-black text-white font-mono">
                <div className="p-8 border border-red-500/20 bg-red-950/10 rounded-lg flex flex-col items-center gap-6 max-w-md text-center w-full">
                    <div className="flex flex-col gap-2">
                        <h1 className="text-xl font-bold text-red-500 uppercase tracking-widest">Sovereign Gate</h1>
                        <p className="text-xs text-red-400/60 uppercase tracking-wider">
                            Identity Verification Required
                        </p>
                    </div>

                    <form onSubmit={handleSubmit} className="w-full flex flex-col gap-4">
                        <div className="flex flex-col gap-1 text-left">
                            <label htmlFor="user-id" className="text-[10px] uppercase tracking-widest text-zinc-500">
                                User ID (Optional)
                            </label>
                            <input
                                id="user-id"
                                type="text"
                                value={userIdInput}
                                onChange={(e) => setUserIdInput(e.target.value)}
                                className="w-full bg-black border border-zinc-800 rounded px-3 py-2 text-sm text-white focus:outline-none focus:border-red-500 transition-colors font-mono"
                                placeholder="pilot"
                                disabled={isVerifying}
                            />
                        </div>

                        <div className="flex flex-col gap-1 text-left">
                            <label htmlFor="sovereign-key" className="text-[10px] uppercase tracking-widest text-zinc-500">
                                Enter Master Key
                            </label>
                            <input
                                id="sovereign-key"
                                type="password"
                                value={keyInput}
                                onChange={(e) => setKeyInput(e.target.value)}
                                className="w-full bg-black border border-zinc-800 rounded px-3 py-2 text-sm text-white focus:outline-none focus:border-red-500 transition-colors font-mono"
                                placeholder="••••••••••••••••"
                                disabled={isVerifying}
                                autoFocus
                            />
                        </div>

                        {error && (
                            <div className="text-xs text-red-500 font-mono bg-red-950/20 p-2 rounded border border-red-900/50">
                                {error}
                            </div>
                        )}

                        <button
                            type="submit"
                            disabled={isVerifying || !keyInput}
                            className="w-full bg-red-900/20 hover:bg-red-900/40 text-red-500 border border-red-900/50 rounded px-4 py-2 text-xs uppercase tracking-widest transition-all disabled:opacity-50 disabled:cursor-not-allowed hover:border-red-500/50"
                        >
                            {isVerifying ? 'Verifying...' : 'Authenticate'}
                        </button>
                    </form>
                </div>
            </div>
        );
    }

    return <>{children}</>;
}
