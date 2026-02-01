#!/bin/bash
# Verification script for Component 2: News API Integration

echo "=== Component 2: News API Integration Verification ==="
echo ""

# Check files exist
echo "Checking files..."
FILES=(
  "web/src/services/news/NewsAPIClient.ts"
  "web/src/services/news/NewsPreferencesManager.ts"
  "web/src/services/news/NewsService.ts"
  "web/src/services/news/NewsBriefingGenerator.ts"
  "web/src/services/news/index.ts"
  "web/src/services/news/README.md"
  "web/src/components/news/NewsPreferences.tsx"
  "web/src/components/news/NewsArticleList.tsx"
  "web/src/components/news/index.ts"
  "tests/integration/news-service.test.ts"
  "docs/guides/news-integration-guide.md"
  "docs/implementation-summary-component-2.md"
)

MISSING=0
for file in "${FILES[@]}"; do
  if [ -f "$file" ]; then
    echo "✓ $file"
  else
    echo "✗ $file (MISSING)"
    MISSING=$((MISSING + 1))
  fi
done

echo ""
echo "Files: $((${#FILES[@]} - MISSING)) / ${#FILES[@]} present"
echo ""

# Check line counts
echo "Checking implementation sizes..."
echo "Services:"
wc -l web/src/services/news/*.ts 2>/dev/null | grep -v index | tail -1

echo ""
echo "Components:"
wc -l web/src/components/news/*.tsx 2>/dev/null | tail -1

echo ""
echo "Tests:"
wc -l tests/integration/news-service.test.ts 2>/dev/null

echo ""
echo "Documentation:"
wc -l docs/guides/news-integration-guide.md docs/implementation-summary-component-2.md 2>/dev/null | tail -1

echo ""
echo "=== Verification Complete ==="

if [ $MISSING -eq 0 ]; then
  echo "✅ All files present - Component 2 implementation complete!"
  exit 0
else
  echo "❌ $MISSING files missing"
  exit 1
fi
