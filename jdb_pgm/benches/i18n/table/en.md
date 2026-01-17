## Pgm-Index Benchmark

Performance comparison of Pgm-Index vs Binary Search with different epsilon values.

<%~ _.perf_tables %>

### Accuracy Comparison: jdb_pgm vs pgm_index

<%~ _.accuracy_table %>

### Build Time Comparison: jdb_pgm vs pgm_index

<%~ _.build_time_table %>

### <%= _.lang.config %>
<%= _.lang.query_count %>: <%= _.config.query_count %>
<%= _.lang.data_sizes %>: <%= _.config.data_sizes.map(s => s.toLocaleString()).join(", ") %>
<%= _.lang.epsilon_values %>: <%= _.config.epsilon_values.join(", ") %>

---

### Epsilon (ε) Explained

*Epsilon (ε) controls the accuracy-speed trade-off:*

*Mathematical definition: ε defines the maximum absolute error between the predicted position and the actual position in the data array. When calling `load(data, epsilon, ...)`, ε guarantees |pred - actual| ≤ ε, where positions are indices within the data array of length `data.len()`.*

*Example: For 1M elements with ε=32, if the actual key is at position 1000:*
- ε=32 predicts position between 968-1032, then checks up to 64 elements
- ε=128 predicts position between 872-1128, then checks up to 256 elements

### Notes
#### What is Pgm-Index?
Pgm-Index (Piecewise Geometric Model Index) is a learned index structure that approximates the distribution of keys with piecewise linear models. It provides O(log ε) search time with guaranteed error bounds, where ε controls the trade-off between memory and speed.

#### Why Compare with Binary Search?
Binary search is the baseline for sorted array lookup. Pgm-Index aims to: match or exceed binary search performance, reduce memory overhead compared to traditional indexes, and provide better cache locality for large datasets.

#### Environment
- OS: <%= _.sys.osName %> (<%= _.sys.arch %>)
- CPU: <%= _.sys.cpu %>
- Cores: <%= _.sys.cores %>
- Memory: <%= _.sys.mem %>GB
- Rust: <%= _.sys.rustVer %>

#### References
- [Pgm-Index Paper](https://doi.org/10.1145/3373718.3394764)
- [Official Pgm-Index Site](https://pgm.di.unipi.it/)
- [Learned Indexes](https://arxiv.org/abs/1712.01208)
