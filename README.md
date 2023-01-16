# EGT on EFG

## Benchmark

- the number of iterations: 100'000
- the error is the last one, not the minimum


| Method                  | KuhnPoker             | LeducHold'em          | LeducHold'em13        |
| ----------------------- | --------------------- | --------------------- | --------------------- |
| CFR                     | 0.3[s], error=1.4e-03 | 6.7[s], error=6.0e-03 | 138[s], error=3.2e-03 |
| CFR+                    | 0.3[s], error=4.4e-06 | 6.6[s], error=1.7e-06 | 138[s], error=3.1e-07 |
| EGT                     | 1.3[s], error=1.1e-05 | 29[s],  error=9.9e-05 | 584[s], error=4.6e-05 |
| EGT-centering           | 1.1[s], error=7.0e-10 | 27[s],  error=1.7e-06 | 536[s], error=9.1e-07 |
| mix(CFR+,EGT-centering) | 1.1[s], error=7.0e-10 | 27[s],  error=1.4e-07 | 519[s], error=3.6e-07 |

