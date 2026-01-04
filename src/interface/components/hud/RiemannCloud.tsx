'use client';

import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Text } from '@react-three/drei';
import { useMarketStore } from '@/lib/stores/market-store';
import { Suspense, useMemo, useRef } from 'react';
import * as THREE from 'three';

function RiemannWave() {
    const physics = useMarketStore((s) => s.physics);
    const riemannState = useMarketStore((s) => s.riemannState);

    // Grid resolution
    const SEGMENTS = 100; // Increased density for larger grid
    const meshRef = useRef<THREE.Points>(null);

    // Color based on state
    const color = useMemo(() => {
        if (riemannState === 'momentum') return new THREE.Color('#00c9ff');
        if (riemannState === 'meanReversion') return new THREE.Color('#ff00ff');
        return new THREE.Color('#888888');
    }, [riemannState]);

    // Animation loop
    useFrame(({ clock }) => {
        if (!meshRef.current) return;

        const t = clock.getElapsedTime();
        const positions = meshRef.current.geometry.attributes.position;

        // Dynamic physics parameters
        const frequency = 0.3 + Math.abs(physics.velocity) * 0.5; // Tuned for larger scale
        const amplitude = 0.8 + physics.entropy;
        const speed = 1.0 + physics.acceleration;

        for (let i = 0; i < positions.count; i++) {
            const x = positions.getX(i);
            const z = positions.getZ(i);

            // Wave function: Superposition of sine waves
            const y =
                Math.sin(x * frequency + t * speed) * amplitude * 0.5 +
                Math.cos(z * frequency * 0.8 + t * speed * 0.7) * amplitude * 0.5;

            positions.setY(i, y);
        }

        positions.needsUpdate = true;
    });

    // Generate initial grid points
    const points = useMemo(() => {
        const temp = [];
        const size = 22; // Doubled size to fill viewport
        const step = size / SEGMENTS;

        for (let x = -size / 2; x < size / 2; x += step) {
            for (let z = -size / 2; z < size / 2; z += step) {
                temp.push(x, 0, z);
            }
        }
        return new Float32Array(temp);
    }, []);

    return (
        <points ref={meshRef}>
            <bufferGeometry>
                <bufferAttribute
                    attach="attributes-position"
                    count={points.length / 3}
                    array={points}
                    itemSize={3}
                />
            </bufferGeometry>
            <pointsMaterial
                size={0.12} // Slightly smaller points for elegance
                color={color}
                transparent
                opacity={0.8}
                sizeAttenuation={true}
                blending={THREE.AdditiveBlending}
            />
        </points>
    );
}

export function RiemannCloud() {
    const riemannState = useMarketStore((s) => s.riemannState);
    const riemannProbability = useMarketStore((s) => s.riemannProbability);

    return (
        <div className="relative w-full h-full">
            <Canvas
                className="w-full h-full bg-black/50"
                camera={{ position: [0, 8, 14], fov: 50 }} // Centered camera, looking down
            >
                <Suspense fallback={null}>
                    {/* Atmospheric Fog */}
                    <fog attach="fog" args={['#000', 5, 35]} />

                    <ambientLight intensity={0.2} />

                    {/* The Wave Function */}
                    <RiemannWave />

                    {/* Floor Context Grid (Faint) */}
                    <gridHelper args={[30, 30, '#333', '#111']} position={[0, -4, 0]} /> {/* Wider grid, lower floor */}

                    <OrbitControls enableZoom={true} enablePan={false} maxPolarAngle={Math.PI / 2} />
                </Suspense>
            </Canvas>

            {/* State Overlay */}
            <div className="absolute top-4 left-4 bg-black/80 px-3 py-2 rounded text-xs select-none pointer-events-none border border-white/10 backdrop-blur-sm">
                <div className="flex items-center gap-2">
                    <div
                        className="w-3 h-3 rounded-full animate-pulse"
                        style={{
                            backgroundColor:
                                riemannState === 'momentum' ? '#00c9ff' :
                                    riemannState === 'meanReversion' ? '#ff00ff' :
                                        '#888888',
                            boxShadow: `0 0 10px ${riemannState === 'momentum' ? '#00c9ff' :
                                riemannState === 'meanReversion' ? '#ff00ff' :
                                    '#888888'
                                }`
                        }}
                    />
                    <div className="flex flex-col">
                        <span className="font-mono font-bold tracking-wider text-white leading-none">
                            RIEMANN WAVE FUNCTION
                        </span>
                        <span className="font-mono text-[10px] text-gray-400 leading-none mt-1">
                            P(State) = {(riemannProbability * 100).toFixed(1)}% | {riemannState.toUpperCase()}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    );
}
