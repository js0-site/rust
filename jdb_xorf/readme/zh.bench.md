## 性能基准

| 库 | 过滤器 | 构建(万ops/s) | 查询(万ops/s) | 内存占用 | 对比 |
| --- | --- | --- | --- | --- | --- |
| jdb | BinaryFuse8 | 4793.58 | 88966.34 | 116.04 KB | 1.69x |
| xorf | BinaryFuse8 | 4049.36 | 52580.65 | 116.04 KB | - |
| jdb | BinaryFuse16 | 4894.69 | 77217.39 | 232.04 KB | 1.57x |
| xorf | BinaryFuse16 | 3756.41 | 49195.52 | 232.04 KB | - |
| jdb | BinaryFuse32 | 4525.89 | 67930.45 | 464.04 KB | 1.43x |
| xorf | BinaryFuse32 | 3976.74 | 47487.04 | 464.04 KB | - |

## 准确率

| 库 | 过滤器 | 假阳率 | 假阴率 |
| --- | --- | --- | --- |
| jdb | BinaryFuse8 | 0.39100% | 0 |
| xorf | BinaryFuse8 | 0.39175% | 0 |
| jdb | BinaryFuse16 | 0.00172% | 0 |
| xorf | BinaryFuse16 | 0.00178% | 0 |
| jdb | BinaryFuse32 | 0.00000% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
