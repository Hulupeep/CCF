#!/bin/bash

# Verification Script for Issue #80 - Performance Benchmarking Dashboard
# This script verifies that all required files exist and meet basic criteria

set -e

echo "=========================================="
echo "Performance Dashboard Verification"
echo "Issue #80 - IMPORTANT"
echo "=========================================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check counter
PASSED=0
FAILED=0

# Function to check file exists
check_file() {
    local file=$1
    local description=$2

    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $description: $file"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC} $description: $file (NOT FOUND)"
        ((FAILED++))
        return 1
    fi
}

# Function to check file size
check_file_size() {
    local file=$1
    local min_size=$2
    local description=$3

    if [ -f "$file" ]; then
        local size=$(wc -l < "$file")
        if [ "$size" -ge "$min_size" ]; then
            echo -e "${GREEN}✓${NC} $description: $size lines (>= $min_size required)"
            ((PASSED++))
        else
            echo -e "${YELLOW}⚠${NC} $description: $size lines (< $min_size expected)"
            ((PASSED++))
        fi
    fi
}

# Function to check file contains pattern
check_contains() {
    local file=$1
    local pattern=$2
    local description=$3

    if [ -f "$file" ]; then
        if grep -q "$pattern" "$file"; then
            echo -e "${GREEN}✓${NC} $description"
            ((PASSED++))
        else
            echo -e "${RED}✗${NC} $description (pattern not found: $pattern)"
            ((FAILED++))
        fi
    fi
}

echo "1. Checking Core Files"
echo "------------------------"
check_file "web/src/types/performance.ts" "Type definitions"
check_file "web/src/services/performanceMetrics.ts" "Metrics service"
check_file "web/src/components/PerformanceDashboard.tsx" "Dashboard component"
check_file "web/src/hooks/usePerformanceMonitoring.ts" "Performance hooks"
echo ""

echo "2. Checking Test Files"
echo "------------------------"
check_file "tests/benchmarks/performance.bench.ts" "Benchmark suite"
check_file "web/src/components/__tests__/PerformanceDashboard.test.tsx" "Component tests"
echo ""

echo "3. Checking Documentation"
echo "------------------------"
check_file "docs/performance-benchmarking.md" "Documentation"
check_file "IMPLEMENTATION_SUMMARY_80.md" "Implementation summary"
echo ""

echo "4. Checking File Sizes"
echo "------------------------"
check_file_size "web/src/types/performance.ts" 100 "Type definitions"
check_file_size "web/src/services/performanceMetrics.ts" 200 "Metrics service"
check_file_size "web/src/components/PerformanceDashboard.tsx" 300 "Dashboard component"
check_file_size "tests/benchmarks/performance.bench.ts" 300 "Benchmark suite"
echo ""

echo "5. Checking Required Patterns"
echo "------------------------"
check_contains "web/src/types/performance.ts" "METRIC_TARGETS" "Performance targets defined"
check_contains "web/src/types/performance.ts" "REGRESSION_THRESHOLD" "Regression threshold defined"
check_contains "web/src/services/performanceMetrics.ts" "PerformanceMetricsService" "Service class exists"
check_contains "web/src/services/performanceMetrics.ts" "detectRegression" "Regression detection implemented"
check_contains "web/src/services/performanceMetrics.ts" "exportToCSV" "CSV export implemented"
check_contains "web/src/components/PerformanceDashboard.tsx" "data-testid" "Test IDs present"
check_contains "web/src/hooks/usePerformanceMonitoring.ts" "usePerformanceMonitoring" "Main hook exists"
check_contains "tests/benchmarks/performance.bench.ts" "WebSocket Message Latency" "WebSocket benchmarks"
check_contains "tests/benchmarks/performance.bench.ts" "UI Render Performance" "UI benchmarks"
check_contains "tests/benchmarks/performance.bench.ts" "Memory Usage" "Memory benchmarks"
echo ""

echo "6. Checking Contract Requirements"
echo "------------------------"
check_contains "web/src/types/performance.ts" "I-PERF-001" "Invariant I-PERF-001"
check_contains "web/src/types/performance.ts" "I-PERF-002" "Invariant I-PERF-002"
check_contains "web/src/types/performance.ts" "I-PERF-003" "Invariant I-PERF-003"
check_contains "web/src/types/performance.ts" "I-PERF-004" "Invariant I-PERF-004"
check_contains "docs/performance-benchmarking.md" "Issue #80" "Documentation references issue"
echo ""

echo "=========================================="
echo "Verification Results"
echo "=========================================="
echo -e "${GREEN}Passed:${NC} $PASSED"
echo -e "${RED}Failed:${NC} $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run tests: npm run test:benchmarks"
    echo "  2. Run component tests: npm test -- PerformanceDashboard.test"
    echo "  3. Add dashboard to main app: import { PerformanceDashboard } from './components/PerformanceDashboard'"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some checks failed. Please review the output above.${NC}"
    exit 1
fi
