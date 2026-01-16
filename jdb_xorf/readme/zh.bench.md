## 性能基准

| 库 | 过滤器 | 构建(万ops/s) | 查询(万ops/s) | 内存占用 | 对比 |
| --- | --- | --- | --- | --- | --- |
| jdb | BinaryFuse8 | 4710.41 | 58981.77 | 116.04 KB | 1.15x |
| xorf | BinaryFuse8 | 3980.48 | 51298.39 | 116.04 KB | - |
| jdb | BinaryFuse16 | 4726.91 | 55813.24 | 232.04 KB | 1.14x |
| xorf | BinaryFuse16 | 3997.96 | 48907.73 | 232.04 KB | - |
| jdb | BinaryFuse32 | 4579.06 | 54709.33 | 464.04 KB | 1.14x |
| xorf | BinaryFuse32 | 3948.46 | 47830.30 | 464.04 KB | - |

## 准确率

| 库 | 过滤器 | 假阳率 | 假阴率 |
| --- | --- | --- | --- |
| jdb | BinaryFuse8 | 0.38930% | 0 |
| xorf | BinaryFuse8 | 0.39077% | 0 |
| jdb | BinaryFuse16 | 0.00150% | 0 |
| xorf | BinaryFuse16 | 0.00146% | 0 |
| jdb | BinaryFuse32 | 0.00000% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
