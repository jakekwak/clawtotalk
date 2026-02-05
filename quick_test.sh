#!/bin/bash
# Quick Test Script for Dioxus Voice Assistant
# This script runs a comprehensive test of the application

set -e  # Exit on error

echo "🚀 Dioxus Voice Assistant - Quick Test"
echo "======================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check if Rust is installed
print_step "Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install from https://rustup.rs/"
    exit 1
fi
print_success "Rust is installed: $(rustc --version)"
echo ""

# Check if Node.js is installed (for server)
print_step "Checking Node.js installation..."
if ! command -v node &> /dev/null; then
    print_warning "Node.js is not installed. Server tests will be skipped."
    print_warning "Install from https://nodejs.org/ to test with server."
    SERVER_AVAILABLE=false
else
    print_success "Node.js is installed: $(node --version)"
    SERVER_AVAILABLE=true
fi
echo ""

# Step 1: Run unit tests
print_step "Step 1: Running unit tests..."
if cargo test --lib --quiet; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi
echo ""

# Step 2: Run integration tests
print_step "Step 2: Running integration tests..."
if cargo test --test integration_tests --quiet; then
    print_success "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi
echo ""

# Step 3: Run property-based tests (this takes longer)
print_step "Step 3: Running property-based tests (this may take a while)..."
if cargo test --test proptest --quiet; then
    print_success "Property-based tests passed"
else
    print_error "Property-based tests failed"
    exit 1
fi
echo ""

# Step 4: Run performance tests
print_step "Step 4: Running performance tests..."
if cargo test --test performance_proptest --quiet; then
    print_success "Performance tests passed"
else
    print_error "Performance tests failed"
    exit 1
fi
echo ""

# Step 5: Build release binary
print_step "Step 5: Building release binary..."
if cargo build --release --quiet; then
    print_success "Release build successful"
    BINARY_SIZE=$(du -h target/release/dioxus-voice-assistant | cut -f1)
    echo "   Binary size: $BINARY_SIZE"
else
    print_error "Release build failed"
    exit 1
fi
echo ""

# Step 6: Test server (if Node.js is available)
if [ "$SERVER_AVAILABLE" = true ]; then
    print_step "Step 6: Testing mock server..."
    
    # Check if node_modules exists
    if [ ! -d "node_modules" ]; then
        print_step "Installing server dependencies..."
        npm install --silent
    fi
    
    # Create a test .env file
    if [ ! -f ".env" ]; then
        print_step "Creating test .env file..."
        cat > .env << EOF
MOCK_MODE=true
PORT=3333
BOT_NAME=TestBot
USE_OPENCLAW=false
EOF
    fi
    
    # Start server in background
    print_step "Starting mock server..."
    node server.js &
    SERVER_PID=$!
    
    # Wait for server to start
    sleep 2
    
    # Test server health endpoint
    if curl -s http://localhost:3333/health > /dev/null; then
        print_success "Mock server is running"
        
        # Get server info
        SERVER_INFO=$(curl -s http://localhost:3333/health)
        echo "   Server info: $SERVER_INFO"
    else
        print_error "Mock server failed to start"
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
    
    # Stop server
    print_step "Stopping mock server..."
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    print_success "Mock server stopped"
    echo ""
else
    print_warning "Step 6: Skipping server tests (Node.js not available)"
    echo ""
fi

# Summary
echo "======================================"
echo -e "${GREEN}✓ All tests passed!${NC}"
echo ""
echo "Next steps:"
echo "  1. Run the app: cargo run --release"
echo "  2. Start the server: npm start (in another terminal)"
echo "  3. Configure the app to connect to http://localhost:3333"
echo ""
echo "For more details, see LOCAL_TESTING_GUIDE.md"
