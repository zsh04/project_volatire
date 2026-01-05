import { useState, useEffect } from 'react';
import { useSystemStore } from '../lib/stores/system-store';

/**
 * Directive-88: Semantic Nullification (The Eraser)
 * 
 * Provides "Fading" opacity when the OODA loop returns a Nullified/Blind state.
 * This prevents the Pilot from internalizing "Ghost Data".
 */
export function useNullification() {
    const { ooda } = useSystemStore();
    const [isNullified, setNullified] = useState(false);
    const [fadeOpacity, setFadeOpacity] = useState(1.0);

    useEffect(() => {
        if (!ooda) return;

        // Detection Logic:
        // If sentiment is missing (undefined/null) but we are supposed to be active,
        // it implies a Firewall Rejection (Blind Mode).
        // Also check if reasoning is empty or explicitly flagged?
        // D-87 implementation sets sentiment to None on rejection.

        // Note: ooda.sentiment_score coming from protobuf is optional/nullable in some generated types,
        // or might default to 0. In our store it is likely `number | undefined`.

        // Detailed check: If we have a timestamp but no sentiment/regime, it's a Nullificaiton.
        const isBlind = ooda.sentiment_score === undefined || ooda.sentiment_score === null;

        if (isBlind) {
            setNullified(true);
            setFadeOpacity(0.3); // "Fade" effect (Ghost Mode)
        } else {
            setNullified(false);
            setFadeOpacity(1.0);
        }

    }, [ooda]);

    return { isNullified, fadeOpacity };
}
