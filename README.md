# EGT on EFG

## Benchmark

| Method                  | KuhnPoker              | LeducHold'em(3 ranks) | LeducHold'em(13 ranks) |
| ----------------------- | ---------------------- | --------------------- | ---------------------  |
| CFR                     | 0.12[s], error=1.4e-03 | 14[s], error=1.8e-03  | 486[s],  error=9.6e-04 |
| CFR+                    | 0.11[s], error=4.3e-06 | 14[s], error=1.7e-07  | 476[s],  error=2.1e-08 |
| EGT                     | 0.24[s], error=3.3e-06 | 40[s], error=2.8e-06  | 1146[s], error=2.0e-06 |
| EGT-centering           | 0.23[s], error=1.9e-09 | 42[s], error=1.7e-07  | 1119[s], error=1.6e-08 |
| EGT-centering with CFR+ | 0.21[s], error=9.7e-10 | 43[s], error=7.0e-10  | 1048[s], error=2.8e-09 |

![kuhn](image/20230201-22:44-kuhn.png)
![leduc](image/20230201-22:45-leduc.png)
![leduc13](image/20230202-00:19-leduc13.png)
