## Performance Benchmark

| 库 | 过滤器 | 构建时间 | 查询时间 | 内存占用 |
| --- | --- | --- | --- | --- |
| jdb | BinaryFuse8 | 1.72 ms | 161.26 μs | 116.04 KB |
| xorf | BinaryFuse8 | 2.49 ms | 189.32 μs | 116.04 KB |
| jdb | BinaryFuse16 | 1.71 ms | 175.21 μs | 232.04 KB |
| xorf | BinaryFuse16 | 2.48 ms | 199.47 μs | 232.04 KB |
| jdb | BinaryFuse32 | 1.79 ms | 181.24 μs | 464.04 KB |
| xorf | BinaryFuse32 | 2.49 ms | 209.04 μs | 464.04 KB |

## Accuracy

| 库 | 过滤器 | 假阳率 | 假阴率 |
| --- | --- | --- | --- |
| jdb | BinaryFuse8 | 0.39206% | 0 |
| xorf | BinaryFuse8 | 0.38810% | 0 |
| jdb | BinaryFuse16 | 0.00166% | 0 |
| xorf | BinaryFuse16 | 0.00176% | 0 |
| jdb | BinaryFuse32 | 0.00000% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
