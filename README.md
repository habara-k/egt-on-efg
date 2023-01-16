# EGT on EFG

## Benchmark

| Method                   | KuhnPoker(step=10^5)  | LeducHold'em(step=10^5) | 
| ------------------------ | --------------------- | ----------------------- | 
| EGT                      | 1.3[s], error=1.1e-05 | 29[s], error=9.9e-05    | 
| CFR                      | 0.3[s], error=1.4e-03 | 6.7[s], error=6.0e-03   | 
| CFR+                     | 0.3[s], error=4.4e-06 | 6.6[s], error=1.7e-06   | 

<!--
| EGT(restarting)          | 35[s], error=3.6e-09 | 197[s], error=1.5e-06   | 
| mix(CFR+,EGT-restarting) | 34[s], error=7.0e-10 | 187[s], error=1.1e-07   | 
-->