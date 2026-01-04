import type { Config } from 'tailwindcss';

const config: Config = {
    content: [
        './app/**/*.{js,ts,jsx,tsx,mdx}',
        './components/**/*.{js,ts,jsx,tsx,mdx}',
        './lib/**/*.{js,ts,jsx,tsx,mdx}',
    ],
    safelist: [
        // Preserve dynamic color classes
        'text-emerald-400',
        'text-red-500',
        'text-yellow-400',
        'text-blue-400',
        'bg-emerald-500',
        'bg-red-500',
        'bg-yellow-500',
        'bg-blue-500',
        // Preserve glow classes
        'shadow-glow-sm',
        'shadow-glow-md',
        'shadow-glow-lg',
        'shadow-glow-red-sm',
        'shadow-glow-red-md',
        // Preserve animation classes
        'animate-pulse-slow',
        'animate-pulse-fast',
        'animate-glow',
        // Preserve glass classes
        'bg-glass',
        'border-glass-border',
        'bg-glass-highlight',
    ],
    theme: {
        extend: {
            fontFamily: {
                sans: ['var(--font-inter)'],
                mono: ['var(--font-mono)'],
            },
            colors: {
                // Kinetic States
                laminar: '#44ff44',
                turbulent: '#ff4444',
                veto: '#ff0000',

                // Physics Gradient
                momentum: '#00c9ff',
                meanReversion: '#ff00ff',

                // UI Layers (Glassmorphism)
                glass: {
                    DEFAULT: 'rgba(0, 0, 0, 0.7)',
                    border: 'rgba(255, 255, 255, 0.1)',
                    highlight: 'rgba(255, 255, 255, 0.05)',
                },
                hud: {
                    black: '#050505',
                    dark: '#0a0a0a',
                    gray: '#111111',
                }
            },
            backgroundImage: {
                'cyber-grid': 'linear-gradient(to right, rgba(255, 255, 255, 0.02) 1px, transparent 1px), linear-gradient(to bottom, rgba(255, 255, 255, 0.02) 1px, transparent 1px)',
                'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
            },
            boxShadow: {
                // Neon Glows
                'glow-sm': '0 0 10px rgba(68, 255, 68, 0.2)',
                'glow-md': '0 0 20px rgba(68, 255, 68, 0.4)',
                'glow-lg': '0 0 30px rgba(68, 255, 68, 0.6)',
                'glow-red-sm': '0 0 10px rgba(255, 68, 68, 0.2)',
                'glow-red-md': '0 0 20px rgba(255, 68, 68, 0.4)',
            },
            animation: {
                'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
                'pulse-fast': 'pulse 1s cubic-bezier(0.4, 0, 0.6, 1) infinite',
                'glow': 'glow 2s ease-in-out infinite alternate',
                'slide-up': 'slideUp 0.3s ease-out forwards',
                'fade-in': 'fadeIn 0.5s ease-out forwards',
            },
            keyframes: {
                glow: {
                    '0%': { boxShadow: '0 0 5px rgba(68, 255, 68, 0.2)' },
                    '100%': { boxShadow: '0 0 20px rgba(68, 255, 68, 0.6)' },
                },
                slideUp: {
                    '0%': { transform: 'translateY(10px)', opacity: '0' },
                    '100%': { transform: 'translateY(0)', opacity: '1' },
                },
                fadeIn: {
                    '0%': { opacity: '0' },
                    '100%': { opacity: '1' },
                },
            },
        },
    },
    plugins: [],
};

export default config;
