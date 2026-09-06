## Performance Benchmark

| Library | Filter | Bf Ops | Query Ops | Memory | Speedup |
| --- | --- | --- | --- | --- | --- |
| jdb | Bf8 | 4930.28 | 75812.43 | 116.04 KB | 1.48x |
| xorf | BinaryFuse8 | 5345.37 | 51182.80 | 116.04 KB | - |
| jdb | Bf16 | 4811.49 | 68210.47 | 232.04 KB | 1.55x |
| xorf | BinaryFuse16 | 5021.32 | 44015.53 | 232.04 KB | - |
| jdb | Bf32 | 4775.74 | 62412.94 | 464.04 KB | 1.29x |
| xorf | BinaryFuse32 | 4591.10 | 48338.36 | 464.04 KB | - |

## Accuracy

| Library | Filter | False Positive Rate | False Negative Rate |
| --- | --- | --- | --- |
| jdb | Bf8 | 0.39076% | 0 |
| xorf | BinaryFuse8 | 0.38816% | 0 |
| jdb | Bf16 | 0.00154% | 0 |
| xorf | BinaryFuse16 | 0.00155% | 0 |
| jdb | Bf32 | 0.00000% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
