#!/bin/bash

# Traverse Performance Benchmarking Script
# This script runs comprehensive performance benchmarks for the traverse CLI

echo "Traverse Performance Benchmarking"
echo "==============================="
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create benchmark directory with static name
BENCHMARK_DIR="benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p "$BENCHMARK_DIR"

echo -e "${YELLOW}Setting up benchmark environment...${NC}"

# Check if traverse CLI is available
TRAVERSE_CLI="./target/release/traverse-cli"
if [ ! -f "$TRAVERSE_CLI" ]; then
    echo -e "${RED}Error: traverse CLI not found at $TRAVERSE_CLI. Please build the project first.${NC}"
    exit 1
fi

echo -e "${YELLOW}Benchmark 1: Storage Key Generation Performance${NC}"
echo "Testing storage key generation speed..."

# Create test ABI file with timestamp
cat > "$BENCHMARK_DIR/test_abi_${TIMESTAMP}.json" << 'EOF'
[
    {
        "type": "function",
        "name": "balanceOf",
        "inputs": [
            {
                "name": "account",
                "type": "address"
            }
        ],
        "outputs": [
            {
                "name": "",
                "type": "uint256"
            }
        ]
    },
    {
        "type": "function",
        "name": "totalSupply",
        "inputs": [],
        "outputs": [
            {
                "name": "",
                "type": "uint256"
            }
        ]
    },
    {
        "type": "function",
        "name": "transfer",
        "inputs": [
            {
                "name": "to",
                "type": "address"
            },
            {
                "name": "amount",
                "type": "uint256"
            }
        ],
        "outputs": [
            {
                "name": "",
                "type": "bool"
            }
        ]
    }
]
EOF

# Benchmark storage key generation
echo "Running storage key generation benchmark..."
start_time=$(date +%s%N)
for i in {1..100}; do
    $TRAVERSE_CLI ethereum compile-layout "$BENCHMARK_DIR/test_abi_${TIMESTAMP}.json" --output "$BENCHMARK_DIR/layout_${TIMESTAMP}_$i.json" --format json > /dev/null 2>&1
done
end_time=$(date +%s%N)

key_gen_time=$(( (end_time - start_time) / 1000000 ))
echo "Storage key generation: 100 iterations in ${key_gen_time}ms"
echo "Average: $((key_gen_time / 100))ms per operation"

# Benchmark 2: Query Resolution Performance
echo ""
echo -e "${YELLOW}Benchmark 2: Query Resolution Performance${NC}"
echo "Testing query resolution speed..."

# Use one of the generated layouts for testing
layout_file="$BENCHMARK_DIR/layout_${TIMESTAMP}_1.json"
if [ -f "$layout_file" ]; then
    echo "Running query resolution benchmark..."
    start_time=$(date +%s%N)
    for i in {1..50}; do
        $TRAVERSE_CLI ethereum resolve-query "_balances[0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C]" "$layout_file" --output "$BENCHMARK_DIR/query_${TIMESTAMP}_$i.json" > /dev/null 2>&1
    done
    end_time=$(date +%s%N)
    
    query_time=$(( (end_time - start_time) / 1000000 ))
    echo "Query resolution: 50 iterations in ${query_time}ms"
    echo "Average: $((query_time / 50))ms per operation"
else
    echo "Layout file not found, skipping query resolution benchmark"
fi

echo ""
echo -e "${YELLOW}Benchmark 3: Circuit Verification Performance${NC}"
echo "Testing circuit verification speed..."

# This would test circuit verification if available
echo "Circuit verification benchmark placeholder"
echo "Note: Full circuit verification requires additional setup"

echo ""
echo -e "${YELLOW}Benchmark 4: Memory Usage Analysis${NC}"
echo "Testing memory usage during operations..."

# Memory usage test
echo "Running memory usage analysis..."
if command -v /usr/bin/time &> /dev/null; then
    echo "Memory usage during layout compilation:"
    /usr/bin/time -v $TRAVERSE_CLI ethereum compile-layout "$BENCHMARK_DIR/test_abi_${TIMESTAMP}.json" --output "$BENCHMARK_DIR/memory_test_${TIMESTAMP}.json" --format json 2>&1 | grep -E "(Maximum resident|Average resident)" || echo "Memory measurements not available"
else
    echo "GNU time not available, skipping memory analysis"
fi

# Create test data for bulk operations
echo ""
echo "Creating test data for bulk operations..."

# Create multiple ABI files
for i in {1..10}; do
    cat > "$BENCHMARK_DIR/test_abi_${TIMESTAMP}_$i.json" << EOF
[
    {
        "type": "function",
        "name": "balanceOf",
        "inputs": [
            {
                "name": "account",
                "type": "address"
            }
        ],
        "outputs": [
            {
                "name": "",
                "type": "uint256"
            }
        ]
    },
    {
        "type": "function",
        "name": "getValue$i",
        "inputs": [],
        "outputs": [
            {
                "name": "",
                "type": "uint256"
            }
        ]
    }
]
EOF
done

# Benchmark bulk operations
echo "Running bulk operations benchmark..."
start_time=$(date +%s%N)
for i in {1..10}; do
    $TRAVERSE_CLI ethereum compile-layout "$BENCHMARK_DIR/test_abi_${TIMESTAMP}_$i.json" --output "$BENCHMARK_DIR/bulk_layout_${TIMESTAMP}_$i.json" --format json > /dev/null 2>&1
done
end_time=$(date +%s%N)

bulk_time=$(( (end_time - start_time) / 1000000 ))
echo "Bulk operations: 10 files in ${bulk_time}ms"
echo "Average: $((bulk_time / 10))ms per file"

echo ""
echo -e "${YELLOW}Benchmark 5: Coprocessor Limits Testing${NC}"
echo "Testing coprocessor input limits..."

# Test large query generation
echo "Testing large query generation..."
large_queries=""
for i in {1..100}; do
    large_queries="${large_queries}balanceOf[0x$(printf '%040x' $i)],"
done

# Remove trailing comma
large_queries=${large_queries%,}

start_time=$(date +%s%N)
$TRAVERSE_CLI ethereum generate-queries "$layout_file" "$large_queries" --output "$BENCHMARK_DIR/large_queries_${TIMESTAMP}.json" --include-examples > /dev/null 2>&1
end_time=$(date +%s%N)

large_query_time=$(( (end_time - start_time) / 1000000 ))
echo "Large query generation: 100 queries in ${large_query_time}ms"

# Test circuit input size limits
echo "Testing circuit input size estimation..."
if [ -f "$BENCHMARK_DIR/large_queries_${TIMESTAMP}.json" ]; then
    query_size=$(wc -c < "$BENCHMARK_DIR/large_queries_${TIMESTAMP}.json")
    echo "Generated query file size: ${query_size} bytes"
    if [ "$query_size" -gt 1000000 ]; then
        echo "Warning: Query file size exceeds 1MB - may hit circuit limits"
    fi
fi

echo ""
echo -e "${YELLOW}Benchmark Results Summary${NC}"
echo "========================="
echo ""
echo "Performance Metrics:"
echo "- Storage Key Generation: $((key_gen_time / 100))ms per operation"
if [ -n "$query_time" ]; then
    echo "- Query Resolution: $((query_time / 50))ms per operation"
fi
echo "- Bulk Operations: $((bulk_time / 10))ms per file"
echo "- Large Query Generation: ${large_query_time}ms for 100 queries"
echo ""
echo "Resource Usage:"
echo "- Test files generated: $(ls -1 "$BENCHMARK_DIR"/*${TIMESTAMP}* 2>/dev/null | wc -l)"
echo "- Total disk usage: $(du -sh "$BENCHMARK_DIR" | cut -f1)"
echo ""

# Generate performance report with timestamp in filename
cat > "$BENCHMARK_DIR/performance_report_${TIMESTAMP}.md" << EOF
# Traverse Performance Benchmark Report

Generated on: $(date)
Benchmark ID: ${TIMESTAMP}

## Performance Metrics

### Storage Key Generation
- **Average Time**: $((key_gen_time / 100))ms per operation
- **Total Time**: ${key_gen_time}ms for 100 operations
- **Throughput**: $((100 * 1000 / key_gen_time)) operations per second

### Query Resolution
$(if [ -n "$query_time" ]; then
    echo "- **Average Time**: $((query_time / 50))ms per operation"
    echo "- **Total Time**: ${query_time}ms for 50 operations"
    echo "- **Throughput**: $((50 * 1000 / query_time)) operations per second"
else
    echo "- **Status**: Skipped (layout file not available)"
fi)

### Bulk Operations
- **Average Time**: $((bulk_time / 10))ms per file
- **Total Time**: ${bulk_time}ms for 10 files
- **Throughput**: $((10 * 1000 / bulk_time)) files per second

### Large Query Generation
- **Time**: ${large_query_time}ms for 100 queries
- **Throughput**: $((100 * 1000 / large_query_time)) queries per second

## Resource Usage

- **Test Files Generated**: $(ls -1 "$BENCHMARK_DIR"/*${TIMESTAMP}* 2>/dev/null | wc -l)
- **Total Disk Usage**: $(du -sh "$BENCHMARK_DIR" | cut -f1)
- **Average File Size**: $(ls -la "$BENCHMARK_DIR"/*${TIMESTAMP}* 2>/dev/null | awk '{total+=$5; count++} END {if(count>0) print total/count " bytes"; else print "N/A"}')

## System Information

- **OS**: $(uname -s -r)
- **Architecture**: $(uname -m)
- **Date**: $(date)
- **Benchmark ID**: ${TIMESTAMP}

## Recommendations

- Storage key generation performance is $(if [ $((key_gen_time / 100)) -lt 50 ]; then echo "excellent"; elif [ $((key_gen_time / 100)) -lt 100 ]; then echo "good"; else echo "needs optimization"; fi)
- Query resolution performance is $(if [ -n "$query_time" ] && [ $((query_time / 50)) -lt 100 ]; then echo "excellent"; elif [ -n "$query_time" ] && [ $((query_time / 50)) -lt 200 ]; then echo "good"; else echo "needs optimization"; fi)
- Bulk operations performance is $(if [ $((bulk_time / 10)) -lt 100 ]; then echo "excellent"; elif [ $((bulk_time / 10)) -lt 200 ]; then echo "good"; else echo "needs optimization"; fi)

EOF

echo "Full report saved to: $BENCHMARK_DIR/performance_report_${TIMESTAMP}.md"
echo ""
echo "Benchmark complete! Results saved to: $BENCHMARK_DIR"
echo "Report files for this run are tagged with: ${TIMESTAMP}" 