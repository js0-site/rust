## Performance Benchmark

| Library | Filter | Bf Ops | Query Ops | Memory | Speedup |
| --- | --- | --- | --- | --- | --- |
| jdb | Bf8 | 4789.53 | 88241.08 | 116.04 KB | - |
| xorf | BinaryFuse16 | 3831.54 | 49478.88 | 232.04 KB | - |
| xorf | BinaryFuse32 | 3968.20 | 48136.63 | 464.04 KB | - |
| jdb | Bf16 | 4734.34 | 76133.28 | 232.04 KB | - |
| jdb | Bf32 | 4737.58 | 67273.21 | 464.04 KB | - |
| xorf | BinaryFuse8 | 3989.56 | 52415.41 | 116.04 KB | - |

## Accuracy

| Library | Filter | False Positive Rate | False Negative Rate |
| --- | --- | --- | --- |
| jdb | Bf8 | 0.39084% | 0 |
| xorf | BinaryFuse16 | 0.00157% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
| jdb | Bf16 | 0.00169% | 0 |
| jdb | Bf32 | 0.00000% | 0 |
| xorf | BinaryFuse8 | 0.38989% | 0 |
