import React, { useRef, useState } from 'react';
import { useFrame } from '@react-three/fiber';
import { Html } from '@react-three/drei';
import * as THREE from 'three';
import { useNullification } from '../../src/hooks/useNullification';

interface ReasoningBubbleProps {
    position: [number, number, number];
    content: string;
    type: string;
    probability: number;
}

export const ReasoningBubble: React.FC<ReasoningBubbleProps> = ({ position, content, type, probability }) => {
    const meshRef = useRef<THREE.Mesh>(null);
    const [hovered, setHovered] = useState(false);
    const { fadeOpacity } = useNullification();

    useFrame((state) => {
        if (meshRef.current) {
            // Bobbing animation
            meshRef.current.position.y = position[1] + Math.sin(state.clock.elapsedTime * 2) * 0.1;
            // Slowly rotate
            meshRef.current.rotation.y += 0.01;
            meshRef.current.rotation.z += 0.005;
        }
    });

    const color = type === 'OBSERVATION' ? '#3B82F6' : // Blue
        type === 'HYPOTHESIS' ? '#F59E0B' : // Amber
            '#10B981'; // Green (Conclusion)

    return (
        <group position={new THREE.Vector3(...position)}>
            <mesh
                ref={meshRef}
                onPointerOver={() => setHovered(true)}
                onPointerOut={() => setHovered(false)}
            >
                <sphereGeometry args={[0.3 + (probability * 0.2), 32, 32]} />
                <meshStandardMaterial
                    color={color}
                    transparent
                    opacity={0.6 * fadeOpacity} // D-88: Fade Effect
                    roughness={0.1}
                    metalness={0.8}
                    emissive={color}
                    emissiveIntensity={(hovered ? 0.8 : 0.2) * fadeOpacity}
                />
            </mesh>

            {/* Connecting Line (Simulated Tail) */}
            <line>
                <bufferGeometry attach="geometry" />
                <lineBasicMaterial attach="material" color={color} transparent opacity={0.3} />
            </line>

            {/* Text Overlay (Visible on Hover or if high probability) */}
            {(hovered || probability > 0.95) && (
                <Html distanceFactor={10} position={[0, 0.5, 0]}>
                    <div className="bg-black/80 backdrop-blur-md border border-white/10 p-2 rounded text-xs font-mono text-white w-48 pointer-events-none select-none shadow-xl transform transition-all duration-200">
                        <div className="flex justify-between items-center mb-1">
                            <span style={{ color }} className="font-bold text-[10px] uppercase">{type}</span>
                            <span className="text-gray-500 text-[10px]">{Math.round(probability * 100)}%</span>
                        </div>
                        <p className="leading-tight text-gray-200">{content}</p>
                    </div>
                </Html>
            )}
        </group>
    );
};
