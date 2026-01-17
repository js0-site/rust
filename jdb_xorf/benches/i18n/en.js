export default {
    // General keys used in regress.js console output
    metric: "Metric",
    build_ops: "Bf(10k ops/s)",
    query_ops: "Query(10k ops/s)",
    memory_kb: "Memory(KB)",
    fp_rate: "FP Rate",
    running_bench: "Running bench-jdb benchmarks...\n",
    saved_history: "Saved to history",

    // SVG Chart titles
    svg_title_main: "Benchmark Results (100K keys)",
    svg_title_build: "Bf Throughput",
    svg_title_query: "Query Throughput",
    svg_title_memory: "Memory Usage",
    svg_title_accuracy: "False Positive Rate",
    svg_no_data: "No data",
    svg_best: "Best",

    // Markdown Table titles & headers
    table_title_perf: "Performance Benchmark",
    table_title_accuracy: "Accuracy",
    table_headers_perf: ["Library", "Filter", "Bf Ops", "Query Ops", "Memory", "Speedup"],
    table_headers_accuracy: ["Library", "Filter", "False Positive Rate", "False Negative Rate"],
};
