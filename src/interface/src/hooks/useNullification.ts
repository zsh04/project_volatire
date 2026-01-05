import { useState, useEffect } from 'react';
import { useSystemStore } from '@/lib/stores/system-store';


/**
 * Directive-88: Semantic Nullification (The Eraser)
 * 
 * Provides "Fading" opacity when the OODA loop returns a Nullified/Blind state.
 * This prevents the Pilot from internalizing "Ghost Data".
 */
export function useNullification() {
    const { systemSanityScore } = useSystemStore();
    const [isNullified, setNullified] = useState(false);
    const [fadeOpacity, setFadeOpacity] = useState(1.0);

    useEffect(() => {
        // D-88: Sanity-based Nullification
        // If system sanity drops below 30%, we consider the "Reasoning" to be hallucinated/blind.
        const isBlind = systemSanityScore < 0.3;

        if (isBlind) {
            setNullified(true);
            setFadeOpacity(0.3); // "Fade" effect (Ghost Mode)
        } else {
            setNullified(false);
            setFadeOpacity(1.0);
        }

    }, [systemSanityScore]);

    return { isNullified, fadeOpacity };
}
