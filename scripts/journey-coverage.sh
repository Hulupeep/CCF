#!/bin/bash
# Journey Coverage Report CLI
# Issue #81: Journey Coverage Report Tool

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
  cat <<EOF
Journey Coverage Report Tool

Usage:
  $0 [command] [options]

Commands:
  generate          Generate all coverage reports (default)
  html              Generate HTML report only
  markdown          Generate Markdown report only
  json              Generate JSON report only
  summary           Print coverage summary to stdout
  check             Check release readiness (exit 0 if ready, 1 if blocked)
  watch             Watch for changes and regenerate reports

Options:
  --run-tests       Run actual journey tests before generating report
  --output <dir>    Specify output directory (default: docs)
  --help            Show this help message

Examples:
  $0 generate
  $0 generate --run-tests
  $0 check
  $0 summary

EOF
}

# Check for required dependencies
check_dependencies() {
  if ! command -v npx &> /dev/null; then
    echo -e "${RED}Error: npx is not available${NC}"
    echo "Please install Node.js and npm"
    exit 1
  fi
}

# Generate reports
generate_reports() {
  local run_tests=""

  if [[ "$1" == "--run-tests" ]]; then
    run_tests="--run-tests"
  fi

  echo -e "${GREEN}Generating journey coverage reports...${NC}"
  cd "$PROJECT_ROOT"

  npx tsx "$SCRIPT_DIR/generate-journey-coverage.ts" $run_tests
}

# Check release readiness
check_readiness() {
  echo -e "${GREEN}Checking release readiness...${NC}"
  cd "$PROJECT_ROOT"

  if npx tsx "$SCRIPT_DIR/generate-journey-coverage.ts" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Release ready: All critical tests passing${NC}"
    exit 0
  else
    echo -e "${RED}✗ Release blocked: Critical tests failing${NC}"
    exit 1
  fi
}

# Print summary
print_summary() {
  cd "$PROJECT_ROOT"
  npx tsx "$SCRIPT_DIR/generate-journey-coverage.ts" 2>&1 | grep -A 100 "Summary:"
}

# Watch for changes
watch_reports() {
  echo -e "${GREEN}Watching for changes...${NC}"
  echo "Press Ctrl+C to stop"

  while true; do
    generate_reports "$@"
    echo -e "\n${YELLOW}Waiting for changes...${NC}"
    sleep 30
  done
}

# Main
main() {
  check_dependencies

  local command="${1:-generate}"
  shift || true

  case "$command" in
    generate)
      generate_reports "$@"
      ;;
    html|markdown|json)
      generate_reports "$@"
      echo -e "${GREEN}Report generated: docs/journey-coverage-report.${command}${NC}"
      ;;
    summary)
      print_summary
      ;;
    check)
      check_readiness
      ;;
    watch)
      watch_reports "$@"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo -e "${RED}Unknown command: $command${NC}"
      usage
      exit 1
      ;;
  esac
}

main "$@"
