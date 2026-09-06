## Performance Benchmark

| Library | Filter | Bf Ops | Query Ops | Memory | Speedup |
| --- | --- | --- | --- | --- | --- |
| jdb | Bf8 | 4849.86 | 71552.43 | 116.04 KB | 1.38x |
| xorf | BinaryFuse8 | 5460.20 | 51882.57 | 116.04 KB | - |
| jdb | Bf16 | 4504.82 | 64713.36 | 232.04 KB | 1.30x |
| xorf | BinaryFuse16 | 5273.35 | 49674.56 | 232.04 KB | - |
| jdb | Bf32 | 4715.24 | 59743.55 | 464.04 KB | 1.22x |
| xorf | BinaryFuse32 | 4668.77 | 48884.67 | 464.04 KB | - |

## Accuracy

| Library | Filter | False Positive Rate | False Negative Rate |
| --- | --- | --- | --- |
| jdb | Bf8 | 0.38702% | 0 |
| xorf | BinaryFuse8 | 0.39045% | 0 |
| jdb | Bf16 | 0.00140% | 0 |
| xorf | BinaryFuse16 | 0.00131% | 0 |
| jdb | Bf32 | 0.00000% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
