#!/bin/bash
# KYX Governance v3.1 Linter (Super-Governance Layer)

REQUIRED_SECTIONS=("## 🧭 Reader Orientation" "## Summary & Prime Directive" "## Analysis & Decisions" "## Invariants & Failure Modes" "## Capability Traceability")
MANDATORY_DOCS=("PROJECT_OVERVIEW.md" "PRD.md" "SAD.md" "TDD.md" "DATABASE_SCHEMA.md" "IMPLEMENTATION_SUMMARY.md" "TEST_PLAN.md" "DEPLOYMENT_GUIDE.md" "OPERATION_GUIDE.md" "MASTER_WORKFLOW_LOG.md" "AI_CONTEXT.md")

MIN_WORDS_ANALYSIS=150 # Reduced from 300 for general validation, but still strict
CONFIDENCE_THRESHOLD=0.7

validate_v3_1() {
    local doc=$1
    echo "🔍 Validating v3.1 Super-Governance: $doc"

    # 1. Header Exact Match
    for section in "${REQUIRED_SECTIONS[@]}"; do
        if ! grep -q "^$section" "$doc"; then
            echo "❌ FAILED: Missing mandatory v3.1 section '$section' in $doc"
            exit 1
        fi
    done

    # 2. Metadata Hardening
    # Check for created_by, ai_prompt, ai_confidence
    if ! grep -q "^created_by:" "$doc"; then echo "❌ FAILED: Missing 'created_by' metadata in $doc"; exit 1; fi
    if ! grep -q "^ai_prompt:" "$doc"; then echo "❌ FAILED: Missing 'ai_prompt' metadata in $doc"; exit 1; fi
    if ! grep -q "^ai_confidence:" "$doc"; then echo "❌ FAILED: Missing 'ai_confidence' metadata in $doc"; exit 1; fi

    # 3. AI Confidence Threshold
    local confidence=$(grep "^ai_confidence:" "$doc" | awk '{print $2}')
    if [[ $(echo "$confidence < $CONFIDENCE_THRESHOLD" | bc -l) -eq 1 ]]; then
        echo "❌ FAILED: ai_confidence ($confidence) is below threshold ($CONFIDENCE_THRESHOLD) in $doc. Requires Human Review."
        exit 1
    fi

    # 4. Word Count (Analysis & Decisions section)
    # Extract text between "## Analysis & Decisions" and the next header
    local analysis_text=$(awk '/^## Analysis & Decisions/{flag=1;next}/^##/{flag=0}flag' "$doc")
    local word_count=$(echo "$analysis_text" | wc -w)
    
    # We only enforce high word count for core docs (PRD, SAD, TDD)
    if [[ "$doc" =~ (PRD|SAD|TDD) ]]; then
        if [ "$word_count" -lt "$MIN_WORDS_ANALYSIS" ]; then
            echo "❌ FAILED: 'Analysis & Decisions' in $doc is too brief ($word_count words). Minimum required: $MIN_WORDS_ANALYSIS words."
            exit 1
        fi
    fi

    # 5. Rule 5: Traceability Check
    if ! grep -q "Capability" "$doc" || ! grep -q "Technical Mechanism" "$doc"; then
        echo "❌ FAILED: Rule 5 violation - Incomplete Traceability Matrix in $doc"
        exit 1
    fi

    # 6. Rule 10: Ambiguity Check
    if grep -Ei "\b(Should|Usually|Maybe)\b" "$doc"; then
        echo "⚠️ WARNING: Rule 10 violation - Ambiguous language detected in $doc"
    fi

    echo "✅ Document $doc is KYX v3.1 COMPLIANT."
}

# 1. Project-wide Doc Set Check (Rule 3)
echo "🔍 Checking Rule 3: Mandatory Document Set..."
DOC_DIR="./docs"
if [ ! -d "$DOC_DIR" ]; then
    echo "❌ FAILED: Mandatory directory '$DOC_DIR' not found."
    exit 1
fi

for doc in "${MANDATORY_DOCS[@]}"; do
    if [ ! -f "$DOC_DIR/$doc" ]; then
        echo "❌ FAILED: Rule 3 violation - Mandatory document '$doc' is missing in $DOC_DIR"
        exit 1
    fi
done
echo "✅ Rule 3: All 11 mandatory documents found."

# 2. Individual Doc Validation
for f in $(find "$DOC_DIR" -maxdepth 1 -name "*.md"); do
    validate_v3_1 "$f"
done
