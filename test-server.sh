#!/bin/bash
# Quick test script for DoggyGallery

set -e

echo "🐕🖼️✨🔒 Testing DoggyGallery..."
echo ""

# Create test media directory
mkdir -p media/subdirectory

# Create a simple test image (1x1 pixel PNG)
echo "Creating test files..."
echo -n "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==" | base64 -d > media/test-image.png
cp media/test-image.png media/subdirectory/nested-image.png

echo "✓ Test files created"
echo ""

# Run with self-signed certificates
echo "Starting server with self-signed certificates..."
echo "  URL: https://127.0.0.1:8443"
echo "  Username: admin"
echo "  Password: admin"
echo ""
echo "Press Ctrl+C to stop"
echo ""

RUST_LOG=doggygallery=info cargo run -- \
    --self-signed-certs-on-the-fly \
    --media-dir ./media \
    --username admin \
    --password admin \
    --host 127.0.0.1 \
    --port 8443
