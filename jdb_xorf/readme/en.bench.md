## Performance Benchmark

| Library | Filter | Bf Ops | Query Ops | Memory | Speedup |
| --- | --- | --- | --- | --- | --- |
| jdb | Bf8 | 4959.87 | 86727.74 | 116.04 KB | 1.65x |
| xorf | BinaryFuse8 | 3971.65 | 52493.29 | 116.04 KB | - |
| jdb | Bf16 | 4809.60 | 76974.42 | 232.04 KB | 1.56x |
| xorf | BinaryFuse16 | 4015.96 | 49222.88 | 232.04 KB | - |
| jdb | Bf32 | 4676.60 | 67297.71 | 464.04 KB | 1.39x |
| xorf | BinaryFuse32 | 4016.81 | 48393.79 | 464.04 KB | - |

## Accuracy

| Library | Filter | False Positive Rate | False Negative Rate |
| --- | --- | --- | --- |
| jdb | Bf8 | 0.39196% | 0 |
| xorf | BinaryFuse8 | 0.38768% | 0 |
| jdb | Bf16 | 0.00132% | 0 |
| xorf | BinaryFuse16 | 0.00165% | 0 |
| jdb | Bf32 | 0.00000% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
