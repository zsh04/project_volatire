/** @type {import('next').NextConfig} */
const nextConfig = {
    // Enable strict mode for better error detection
    reactStrictMode: true,



    // Experimental features
    experimental: {
        // Enable server actions
        serverActions: {
            bodySizeLimit: '2mb',
        },
    },

    // Headers for gRPC-web CORS
    async headers() {
        return [
            {
                source: '/api/:path*',
                headers: [
                    { key: 'Access-Control-Allow-Origin', value: '*' },
                    { key: 'Access-Control-Allow-Methods', value: 'GET,POST,OPTIONS' },
                    { key: 'Access-Control-Allow-Headers', value: 'Content-Type, x-grpc-web, x-user-agent' },
                ],
            },
        ];
    },

    // Webpack config for Web Workers is now handled natively by Next.js 14+
    // using new Worker(new URL('./worker.ts', import.meta.url))
};

export default nextConfig;
