#!/bin/bash

# xlsx2sql Release Script
# Usage: ./release.sh [version]
# Example: ./release.sh 0.1.0

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if version is provided
if [ -z "$1" ]; then
    print_error "Version number required!"
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

VERSION="$1"
TAG="v$VERSION"

print_info "Starting release process for version $VERSION"

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "Not in a git repository!"
    exit 1
fi

# Check if there are uncommitted changes
if ! git diff-index --quiet HEAD --; then
    print_warning "You have uncommitted changes!"
    read -p "Do you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Aborting release."
        exit 1
    fi
fi

# Check if tag already exists
if git tag -l | grep -q "^$TAG$"; then
    print_error "Tag $TAG already exists!"
    exit 1
fi

# Update version in Cargo.toml
print_info "Updating version in Cargo.toml..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
else
    # Linux
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
fi

# Update version in main.rs
print_info "Updating version in src/main.rs..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/#\[command(version = \".*\")\]/#[command(version = \"$VERSION\")]/" src/main.rs
else
    # Linux
    sed -i "s/#\[command(version = \".*\")\]/#[command(version = \"$VERSION\")]/" src/main.rs
fi

# Build and test
print_info "Running tests..."
if ! cargo test; then
    print_error "Tests failed! Aborting release."
    exit 1
fi

print_info "Building release binary..."
if ! cargo build --release; then
    print_error "Build failed! Aborting release."
    exit 1
fi

# Commit version changes
print_info "Committing version changes..."
git add Cargo.toml src/main.rs Cargo.lock
git commit -m "Bump version to $VERSION"

# Create and push tag
print_info "Creating tag $TAG..."
git tag -a "$TAG" -m "Release $VERSION"

print_info "Pushing changes and tag..."
git push origin master
git push origin "$TAG"

print_success "Release $VERSION created successfully!"
print_info "GitHub Actions will now build and create the release."
print_info "Check the Actions tab on GitHub: https://github.com/nominalrune/xlsx2sql/actions"

echo
print_info "Release summary:"
echo "  📦 Version: $VERSION"
echo "  🏷️  Tag: $TAG"
echo "  🚀 Status: Triggered GitHub Actions build"
echo "  🔗 Releases: https://github.com/nominalrune/xlsx2sql/releases"
