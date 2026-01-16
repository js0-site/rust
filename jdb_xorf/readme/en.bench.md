## Performance Benchmark

| Library | Filter | Build Ops | Query Ops | Memory | Speedup |
| --- | --- | --- | --- | --- | --- |
| jdb | BinaryFuse8 | 4793.58 | 88966.34 | 116.04 KB | 1.69x |
| xorf | BinaryFuse8 | 4049.36 | 52580.65 | 116.04 KB | - |
| jdb | BinaryFuse16 | 4894.69 | 77217.39 | 232.04 KB | 1.57x |
| xorf | BinaryFuse16 | 3756.41 | 49195.52 | 232.04 KB | - |
| jdb | BinaryFuse32 | 4525.89 | 67930.45 | 464.04 KB | 1.43x |
| xorf | BinaryFuse32 | 3976.74 | 47487.04 | 464.04 KB | - |

## Accuracy

| Library | Filter | False Positive Rate | False Negative Rate |
| --- | --- | --- | --- |
| jdb | BinaryFuse8 | 0.39100% | 0 |
| xorf | BinaryFuse8 | 0.39175% | 0 |
| jdb | BinaryFuse16 | 0.00172% | 0 |
| xorf | BinaryFuse16 | 0.00178% | 0 |
| jdb | BinaryFuse32 | 0.00000% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
