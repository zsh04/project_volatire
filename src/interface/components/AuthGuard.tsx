import React, { useEffect, useState } from 'react';

const BYPASS_KEY = process.env.NEXT_PUBLIC_SOVEREIGN_BYPASS;

interface AuthGuardProps {
    children: React.ReactNode;
}

export function AuthGuard({ children }: AuthGuardProps) {
    const [isAuthenticated, setIsAuthenticated] = useState(false);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        // Check for Sovereign Bypass
        if (BYPASS_KEY) {
            console.log('🛡️ SOVEREIGN BYPASS ACTIVE');
            setIsAuthenticated(true);
            setIsLoading(false);
            return;
        }

        // TODO: Implement actual Sovereign Key challenge here if bypass is not present
        // For now, default to false to block access if no bypass
        setIsAuthenticated(false);
        setIsLoading(false);
    }, []);

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
                <div className="p-8 border border-red-500/20 bg-red-950/10 rounded-lg flex flex-col items-center gap-4 max-w-md text-center">
                    <h1 className="text-xl font-bold text-red-500 uppercase tracking-widest">Access Denied</h1>
                    <p className="text-sm text-red-400/60">
                        Sovereign Identity Verification Failed.
                        <br />
                        Please provide valid credentials or check environment configuration.
                    </p>
                </div>
            </div>
        );
    }

    return <>{children}</>;
}
