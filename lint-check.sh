#!/bin/bash

# Detailed linting and code quality check
# Use this when you want to review and fix code quality issues

set -e

echo "🔍 NodeFoundry Code Quality & Linting Check"
echo "============================================"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo ""
echo -e "${BLUE}🧹 Running Clippy (Detailed Linting)${NC}"
echo "--------------------------------------"
echo "This will show suggestions to improve code quality..."
echo ""

# Run clippy with suggestions but not as errors
cargo clippy --all-targets --all-features -- \
    -W clippy::all \
    -W clippy::pedantic \
    -A clippy::too_many_arguments \
    -A clippy::module_name_repetitions \
    -A clippy::missing_errors_doc \
    -A clippy::missing_panics_doc

echo ""
echo -e "${BLUE}📊 Code Statistics${NC}"
echo "--------------------------------------"
echo "Lines of code:"
find contracts -name "*.rs" -not -path "*/target/*" | xargs wc -l | tail -1

echo ""
echo "Test files:"
find contracts -name "test.rs" | wc -l | awk '{print $1 " test files"}'

echo ""
echo -e "${GREEN}✅ Linting check completed!${NC}"
echo ""
echo "Note: Clippy suggestions are recommendations for code improvement."
echo "They don't prevent compilation or functionality."
