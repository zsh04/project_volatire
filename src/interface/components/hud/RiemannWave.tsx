'use client';

import React, { useRef, useMemo } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Sphere, MeshDistortMaterial } from '@react-three/drei';
import * as THREE from 'three';
import { useSystemStore } from '@/lib/stores/system-store';

function DriftIndicator() {
    const drift = useSystemStore(state => state.audit.driftScore);
    const gapPct = (drift * 100).toFixed(3);
    const color = drift > 0.1 ? 'text-red-500' : drift > 0.05 ? 'text-amber-500' : 'text-emerald-500';

    return (
        <span className={`text-xs font-mono font-bold ${color}`}>
            {drift > 0 ? '+' : ''}{gapPct}%
        </span>
    );
}

/**
 * Directive-UX: Kinetic Core (Riemann Wave)
 * 
 * Transforming into a "Vector Compass" indicating:
 * - Velocity ($v$): Rotation Speed (based on Boyd Hz)
 * - Jerk ($j$): Distortion/Noise (based on Sanity Score)
 * - Regime: Color (Green/Laminar, Amber/Degraded, Red/Critical)
 */

function VectorCompass() {
    const meshRef = useRef<THREE.Mesh>(null!);
    const { nodes, systemSanityScore } = useSystemStore((state) => ({
        nodes: state.nodes,
        systemSanityScore: state.systemSanityScore
    }));

    // Derived Metrics
    // Boyd Hz (Loop Frequency) -> Rotation Speed
    const loopHz = typeof nodes?.boyd?.metricValue === 'number' ? nodes.boyd.metricValue : 1;
    const velocity = Math.max(0.2, loopHz / 10); // Scale 14Hz -> 1.4 speed

    // Sanity -> Distortion (Lower sanity = Higher distortion)
    // Sanity 1.0 -> Distort 0.1
    // Sanity 0.5 -> Distort 0.6
    const distortion = Math.max(0.1, (1.0 - systemSanityScore) * 1.5);

    // Regime Colors
    const color = useMemo(() => {
        if (systemSanityScore > 0.8) return '#00ff41'; // Laminar Green
        if (systemSanityScore > 0.5) return '#ffaa00'; // Degraded Amber
        return '#ff0055'; // Critical Red
    }, [systemSanityScore]);

    useFrame((state, delta) => {
        if (meshRef.current) {
            // Rotation based on Velocity
            meshRef.current.rotation.x += delta * velocity * 0.5;
            meshRef.current.rotation.y += delta * velocity;
        }
    });

    return (
        <Sphere args={[1, 64, 64]} ref={meshRef}>
            <MeshDistortMaterial
                color={color}
                envMapIntensity={1}
                clearcoat={1}
                clearcoatRoughness={0}
                metalness={0.5}
                roughness={0.2}
                distort={distortion}
                speed={velocity * 2} // Jitter speed
            />
        </Sphere>
    );
}

function GridFloor() {
    return (
        <gridHelper
            args={[20, 20, 0x222222, 0x111111]}
            position={[0, -2, 0]}
            rotation={[0, 0, 0]}
        />
    );
}

export function RiemannWave() {
    return (
        <div className="w-full h-full relative">
            <div className="absolute top-4 left-4 z-10 pointer-events-none">
                <h3 className="text-xs font-mono font-bold text-white/40 tracking-widest uppercase">KINETIC CORE</h3>
                {/* D-106 Alpha Gap Overlay */}
                <div className="mt-1 flex items-center gap-2">
                    <span className="text-[10px] text-white/30 font-mono">ALPHA GAP (DRIFT)</span>
                    <DriftIndicator />
                </div>
            </div>

            <Canvas camera={{ position: [0, 0, 4], fov: 45 }}>
                <ambientLight intensity={0.5} />
                <pointLight position={[10, 10, 10]} intensity={1} />
                <spotLight position={[-10, -10, -10]} intensity={0.5} />

                <React.Suspense fallback={null}>
                    <VectorCompass />
                    <GridFloor />
                </React.Suspense>

                <OrbitControls enableZoom={false} enablePan={false} autoRotate={false} />
            </Canvas>
        </div>
    );
}
