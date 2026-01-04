import { NextRequest } from 'next/server';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

const ENVOY_URL = 'http://localhost:8080';

export async function GET(request: NextRequest) {
    return handleProxy(request);
}

export async function POST(request: NextRequest) {
    return handleProxy(request);
}

export async function OPTIONS(request: NextRequest) {
    // Handle CORS preflight
    return new Response(null, {
        status: 204,
        headers: {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
            'Access-Control-Allow-Headers': 'Content-Type, x-grpc-web, x-user-agent, grpc-timeout',
            'Access-Control-Max-Age': '86400',
        },
    });
}

async function handleProxy(request: NextRequest) {
    const path = request.nextUrl.searchParams.get('path') || '';
    const url = `${ENVOY_URL}${path}`;

    try {
        const headers: Record<string, string> = {};
        request.headers.forEach((value, key) => {
            if (!key.startsWith('host') && !key.startsWith('connection')) {
                headers[key] = value;
            }
        });

        const body = request.method === 'POST' ? await request.arrayBuffer() : undefined;

        const response = await fetch(url, {
            method: request.method,
            headers,
            body: body ? Buffer.from(body) : undefined,
        });

        const responseBody = await response.arrayBuffer();

        return new Response(responseBody, {
            status: response.status,
            headers: {
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Expose-Headers': 'grpc-status, grpc-message, grpc-status-details-bin',
                'Content-Type': response.headers.get('content-type') || 'application/grpc-web+proto',
                'grpc-status': response.headers.get('grpc-status') || '0',
                'grpc-message': response.headers.get('grpc-message') || '',
            },
        });
    } catch (error: any) {
        console.error('[gRPC Proxy Error]:', error);
        return new Response(JSON.stringify({ error: error.message }), {
            status: 500,
            headers: {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
            },
        });
    }
}
