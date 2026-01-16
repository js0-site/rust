## Performance Benchmark

| Library | Filter | Bf Ops | Query Ops | Memory | Speedup |
| --- | --- | --- | --- | --- | --- |
| xorf | Bf8 | 3985.84 | 52562.99 | 116.04 KB | - |
| xorf | BinaryFuse16 | 4036.86 | 48915.75 | 232.04 KB | - |
| xorf | BinaryFuse32 | 3931.59 | 48225.88 | 464.04 KB | - |
| jdb | Bf16 | 4599.84 | 76251.81 | 232.04 KB | - |
| jdb | Bf32 | 4767.73 | 66797.11 | 464.04 KB | - |
| jdb | Bf8 | 4742.57 | 86450.03 | 116.04 KB | - |

## Accuracy

| Library | Filter | False Positive Rate | False Negative Rate |
| --- | --- | --- | --- |
| xorf | Bf8 | 0.39252% | 0 |
| xorf | BinaryFuse16 | 0.00157% | 0 |
| xorf | BinaryFuse32 | 0.00000% | 0 |
| jdb | Bf16 | 0.00144% | 0 |
| jdb | Bf32 | 0.00000% | 0 |
| jdb | Bf8 | 0.39059% | 0 |
