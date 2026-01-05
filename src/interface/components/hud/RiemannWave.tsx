'use client';

import { useRef, useMemo } from 'react';
import { Canvas, useFrame, extend } from '@react-three/fiber';
import { shaderMaterial } from '@react-three/drei';
import * as THREE from 'three';
import { useMarketStore } from '@/lib/stores/market-store';
import { useAgentStore } from '@/lib/stores/agent-store';
import { useSystemStore } from '@/lib/stores/system-store'; // D-81
import { ReasoningBubble } from './ReasoningBubble'; // D-81

// -----------------------------------------------------------------------------
// GLSL Shaders
// -----------------------------------------------------------------------------

const vertexShader = `
  uniform float uTime;
  uniform float uVelocity;
  uniform float uEntropy; // Used for amplitude
  uniform float uVeto;    // 0.0 = Normal, 1.0 = Collapse

  varying vec2 vUv;
  varying float vElevation;

  void main() {
    vUv = uv;
    vec3 pos = position;

    // Physics mapping
    float frequency = 1.0 + (uVelocity * 0.5); // Higher velocity = High frequency
    float amplitude = 0.2 + (uEntropy * 1.5);  // Higher entropy = High amplitude (chaos)
    float speed = 1.5;

    // Wave Superposition (Fourier-ish)
    float activeWave = sin(pos.x * frequency + uTime * speed) 
                     * sin(pos.z * frequency * 0.5 + uTime * speed * 0.8) 
                     * amplitude;

    // Veto Collapse: Flatten the wave 
    float collapsedWave = 0.0;
    float elevation = mix(activeWave, collapsedWave, uVeto);

    pos.y += elevation;
    vElevation = elevation;

    gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
  }
`;

const fragmentShader = `
  uniform float uTime;
  uniform float uVeto;
  
  varying float vElevation;
  varying vec2 vUv;

  void main() {
    // Cyberpunk Gradient Palettes
    vec3 colorLow = vec3(0.0, 0.85, 1.0);  // Cyan (Momentum)
    vec3 colorHigh = vec3(0.8, 0.0, 1.0);  // Magenta (Mean Reversion)
    vec3 colorVeto = vec3(1.0, 0.0, 0.0);  // Red (Danger)

    // Map elevation to color mixing
    // Elevation is roughly -2.0 to 2.0 depending on entropy
    float mixStrength = smoothstep(-1.0, 1.0, vElevation);
    
    vec3 color = mix(colorLow, colorHigh, mixStrength);

    // Grid / Wireframe effect logic (Screen space derivative)
    // Simple glowing center for now
    float glow = 1.0 - distance(vUv, vec2(0.5));
    color += glow * 0.2;

    // Phase interference pattern (Moire)
    float interference = sin(vElevation * 10.0 + uTime);
    color += interference * 0.05;

    // Veto Override
    color = mix(color, colorVeto, uVeto);

    gl_FragColor = vec4(color, 0.9);
  }
`;

// -----------------------------------------------------------------------------
// Material Definition
// -----------------------------------------------------------------------------

const WaveMaterial = shaderMaterial(
    {
        uTime: 0,
        uVelocity: 0,
        uEntropy: 0,
        uVeto: 0,
    },
    vertexShader,
    fragmentShader
);

extend({ WaveMaterial });

// Add types for the custom shader material
declare global {
    namespace JSX {
        interface IntrinsicElements {
            waveMaterial: any;
        }
    }
}

// -----------------------------------------------------------------------------
// Component
// -----------------------------------------------------------------------------

function WaveMesh() {
    const materialRef = useRef<any>(null);
    const meshRef = useRef<THREE.Mesh>(null);

    // Connect to stores
    const velocity = useMarketStore(s => s.physics.velocity);
    const entropy = useMarketStore(s => s.physics.entropy);
    const isVetoActive = useAgentStore(s => s.vetoActive);

    // Veto interpolation state
    const vetoLerp = useRef(0);

    useFrame((state, delta) => {
        if (materialRef.current) {
            // Time
            materialRef.current.uTime = state.clock.elapsedTime;

            // LERP Physics for smooth transitions (Liquid feel)
            materialRef.current.uVelocity = THREE.MathUtils.lerp(materialRef.current.uVelocity, Math.abs(velocity), delta * 2);
            materialRef.current.uEntropy = THREE.MathUtils.lerp(materialRef.current.uEntropy, entropy, delta * 2);

            // Veto Collapse Animation
            const targetVeto = isVetoActive ? 1.0 : 0.0;
            vetoLerp.current = THREE.MathUtils.lerp(vetoLerp.current, targetVeto, delta * 5);
            materialRef.current.uVeto = vetoLerp.current;
        }

        // Auto-rotation (Replaces OrbitControls)
        if (meshRef.current) {
            // Rotate around global Y axis (which is local Z because of the -PI/2 X rotation)
            meshRef.current.rotation.z += delta * 0.2;
        }

        // D-84: Track FPS for stress testing
        updateFPSCounter();
    });

    return (
        <group>
            <mesh ref={meshRef} rotation={[-Math.PI / 2, 0, 0]} position={[0, -2, 0]}>
                {/* High segment count for smooth vertex displacement */}
                <planeGeometry args={[30, 30, 128, 128]} />
                {/* @ts-ignore */}
                <waveMaterial
                    ref={materialRef}
                    wireframe={true} // Wireframe looks more "Tactical"
                    transparent={true}
                    side={THREE.DoubleSide}
                />
            </mesh>

            {/* Directive-81: Reasoning Bubbles */}
            {/* We map multiple bubbles if they exist. 
                Position logic:
                x: Spread out horizontally based on index (simulated time)
                y: Float above wave
                z: Slightly randomized depth
            */}
            <ReasoningBubblesOverlay />
        </group>
    );
}

function ReasoningBubblesOverlay() {
    const trace = useSystemStore(s => s.reasoningTrace);

    return (
        <group position={[0, 0, 0]}>
            {trace.map((step, i) => {
                // visual offset: new thoughts appear on right (+x), older on left (-x)
                // This mimics the "feed" moving left.
                // We assume the trace is ordered [oldest, ..., newest] 
                // but usually we want newest on right.

                // Let's assume max 5 items. 
                // i=0 (oldest) -> x = -4
                // i=4 (newest) -> x = 4

                const xPos = -4 + (i * 2.5);

                return (
                    <ReasoningBubble
                        key={step.id || i}
                        position={[xPos, 1.5 + (Math.sin(i) * 0.5), (i % 2 === 0 ? 1 : -1)]}
                        content={step.content}
                        type={step.type}
                        probability={step.probability}
                    />
                )
            })}
        </group>
    );
}

export function RiemannWave() {
    const riemannState = useMarketStore((s) => s.riemannState);
    const riemannProb = useMarketStore((s) => s.riemannProbability);

    return (
        <div className="relative w-full h-full group">
            {/* 3D Scene */}
            <Canvas
                className="w-full h-full bg-gradient-to-b from-[#0D1117] to-black"
                camera={{ position: [0, 8, 12], fov: 45 }}
            >
                <WaveMesh />
            </Canvas>

            {/* HUD Overlay */}
            <div className="absolute top-4 left-4 pointer-events-none select-none">
                <h3 className="text-xs font-bold text-white/40 tracking-widest uppercase mb-1">
                    Riemann Phase-Space
                </h3>
                <div className="flex items-center gap-2">
                    <div className="text-2xl font-mono font-bold text-white">
                        Ψ(x)
                    </div>
                    <div className="flex flex-col">
                        <span className="text-[10px] font-mono text-cyan-400">
                            {riemannState.toUpperCase()}
                        </span>
                        <span className="text-[10px] font-mono text-white/50">
                            P(State) = {(riemannProb * 100).toFixed(1)}%
                        </span>
                    </div>
                </div>
            </div>
        </div>
    );
}

// D-84: FPS Measurement for Stress Testing
let fpsFrames = 0;
let fpsStartTime = performance.now();
let currentFPS = 0;

export function resetFPSCounter() {
    fpsFrames = 0;
    fpsStartTime = performance.now();
    currentFPS = 0;
}

export function getCurrentFPS(): number {
    return currentFPS;
}

export function updateFPSCounter() {
    fpsFrames++;
    const currentTime = performance.now();
    const elapsed = currentTime - fpsStartTime;

    // Update FPS every second
    if (elapsed >= 1000) {
        currentFPS = Math.round((fpsFrames / elapsed) * 1000);
        fpsFrames = 0;
        fpsStartTime = currentTime;
    }
}
