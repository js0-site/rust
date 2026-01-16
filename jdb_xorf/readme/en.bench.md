## Performance Benchmark

| Library | Filter | Bf Ops | Query Ops | Memory | Speedup |
| --- | --- | --- | --- | --- | --- |
| jdb | BinaryFuse8 | 4815.38 | 87659.01 | 116.04 KB | 1.68x |
| xorf | BinaryFuse8 | 3974.81 | 52299.44 | 116.04 KB | - |
| jdb | BinaryFuse16 | 4791.46 | 76409.67 | 232.04 KB | 1.55x |
| xorf | BinaryFuse16 | 3986.36 | 49376.81 | 232.04 KB | - |
| jdb | BinaryFuse32 | 4715.07 | 67206.33 | 464.04 KB | 1.40x |
| xorf | BinaryFuse32 | 3908.10 | 48113.90 | 464.04 KB | - |
| jdb | Bf16 | 4573.49 | 74126.66 | 232.04 KB | - |
| jdb | Bf32 | 4596.82 | 67651.90 | 464.04 KB | - |
| jdb | Bf8 | 4621.66 | 85528.10 | 116.04 KB | - |

## Accuracy

| Library | Filter | False Positive Rate | False Negative Rate |
| --- | --- | --- | --- |
| jdb | BinaryFuse8 | 0.39232% | 0 |
| xorf | BinaryFuse8 | 0.39089% | 0 |
| jdb | BinaryFuse16 | 0.00184% | 0 |
| xorf | BinaryFuse16 | 0.00132% | 0 |
| jdb | BinaryFuse32 | 0.00000% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
| jdb | Bf16 | 0.00164% | 0 |
| jdb | Bf32 | 0.00000% | 0 |
| jdb | Bf8 | 0.38999% | 0 |
