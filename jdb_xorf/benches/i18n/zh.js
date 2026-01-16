export default {
    // General keys used in regress.js console output
    metric: "指标",
    build_ops: "构建(万ops/s)",
    query_ops: "查询(万ops/s)",
    memory_kb: "内存(KB)",
    fp_rate: "假阳率",
    running_bench: "运行 bench-jdb 特性的性能测试...\n",
    saved_history: "已保存到历史记录",

    // SVG Chart titles
    svg_title_main: "基准测试结果 (10万个键)",
    svg_title_build: "构建吞吐量",
    svg_title_query: "查询吞吐量",
    svg_title_memory: "内存占用",
    svg_title_accuracy: "假阳率",
    svg_no_data: "无数据",
    svg_best: "最佳",

    // Markdown Table titles & headers
    table_title_perf: "性能基准",
    table_title_accuracy: "准确率",
    table_headers_perf: ["库", "过滤器", "构建(万ops/s)", "查询(万ops/s)", "内存占用", "对比"],
    table_headers_accuracy: ["库", "过滤器", "假阳率", "假阴率"],
};
