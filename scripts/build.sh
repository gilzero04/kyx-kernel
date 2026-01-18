#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# Kyx Kernel - Docker Build Script with Version Metadata
# ═══════════════════════════════════════════════════════════════════════════════
# Usage: ./scripts/build.sh [TAG]
# Example: ./scripts/build.sh 1.0.0
# ═══════════════════════════════════════════════════════════════════════════════

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Extract version from Cargo.toml if not provided
VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)}"
BUILD_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Image name
IMAGE_NAME="kyx/kernel"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Building Kyx Kernel Docker Image${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "  Version:    ${GREEN}${VERSION}${NC}"
echo -e "  Build Date: ${GREEN}${BUILD_DATE}${NC}"
echo -e "  Git Commit: ${GREEN}${GIT_COMMIT}${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Build Docker image
docker build \
  --build-arg APP_VERSION="${VERSION}" \
  --build-arg BUILD_DATE="${BUILD_DATE}" \
  --build-arg GIT_COMMIT="${GIT_COMMIT}" \
  -t "${IMAGE_NAME}:${VERSION}" \
  -t "${IMAGE_NAME}:latest" \
  .

echo ""
echo -e "${GREEN}✅ Build complete!${NC}"
echo -e "   Image: ${IMAGE_NAME}:${VERSION}"
echo -e "   Image: ${IMAGE_NAME}:latest"
