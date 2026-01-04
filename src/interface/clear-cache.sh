#!/bin/bash
# CSS Hard Reset Script for Next.js

echo "🧹 Clearing Next.js build cache..."
rm -rf .next

echo "🧹 Clearing node_modules/.cache..."
rm -rf node_modules/.cache

echo "✅ Cache cleared!"
echo "🔄 Restart your dev server with: npm run dev"
