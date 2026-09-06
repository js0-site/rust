## Pgm 索引评测

Pgm-Index 与二分查找在不同 epsilon 值下的性能对比。

<%~ \_.perf_tables %>

### 精度对比: jdb_pgm vs pgm_index

<%~ \_.accuracy_table %>

### 构建时间对比: jdb_pgm vs pgm_index

<%~ \_.build_time_table %>

### <%= \_.lang.config %>

<%= _.lang.query_count %>: <%= _.config.query*count %>
<%= *.lang.data*sizes %>: <%= *.config.data*sizes.map(s => s.toLocaleString()).join(", ") %>
<%= *.lang.epsilon*values %>: <%= *.config.epsilon_values.join(", ") %>

---

### Epsilon (ε) 说明

_Epsilon (ε) 控制精度与速度的权衡：_

_数学定义：ε 定义了预测位置与实际位置在数据数组中的最大绝对误差。调用 `load(data, epsilon, ...)` 时，ε 保证 |pred - actual| ≤ ε，其中位置是长度为 `data.len()` 的数据数组中的索引。_

_举例说明：对于 100 万个元素，ε=32 时，如果实际键在位置 1000：_

- ε=32 预测位置在 968-1032 之间，然后检查最多 64 个元素
- ε=128 预测位置在 872-1128 之间，然后检查最多 256 个元素

### 备注

#### 什么是 Pgm-Index?

Pgm-Index（分段几何模型索引）是一种学习型索引结构，使用分段线性模型近似键的分布。它提供 O(log ε) 的搜索时间，并保证误差边界，其中 ε 控制内存和速度之间的权衡。

#### 为什么与二分查找对比?

二分查找是已排序数组查找的基准。Pgm-Index 旨在：匹配或超过二分查找的性能，相比传统索引减少内存开销，为大数据集提供更好的缓存局部性。

#### 环境

- 系统: <%= _.sys.osName %> (<%= _.sys.arch %>)
- CPU: <%= \_.sys.cpu %>
- 核心数: <%= \_.sys.cores %>
- 内存: <%= \_.sys.mem %>GB
- Rust 版本: <%= \_.sys.rustVer %>

#### 参考

- [Pgm-Index 论文](https://doi.org/10.1145/3373718.3394764)
- [Pgm-Index 官方网站](https://pgm.di.unipi.it/)
- [学习型索引](https://arxiv.org/abs/1712.01208)
