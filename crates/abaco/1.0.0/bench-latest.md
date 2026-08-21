# Benchmark Results — Last 3 Runs

| Benchmark | 26-03-27 (a164c6e) | 26-03-27 (a164c6e) | 26-03-27 (e5dac8d) |
|-----------|------|------|------|
| eval_simple/addition | 65.88 ns | 69.03 ns | 70.86 ns |
| eval_simple/mixed_ops | 145.01 ns | 155.16 ns | 153.09 ns |
| eval_simple/parentheses | 162.52 ns | 171.67 ns | 170.23 ns |
| eval_simple/division | 77.01 ns | 72.89 ns | 74.19 ns |
| eval_functions/sqrt | 109.16 ns | 113.38 ns | 115.37 ns |
| eval_functions/sin | 131.78 ns | 136.42 ns | 127.32 ns |
| eval_functions/log2 | 118.48 ns | 129.89 ns | 122.84 ns |
| eval_functions/pow | 231.88 ns | 215.72 ns | 434.60 ns |
| eval_functions/min | 208.39 ns | 209.67 ns | 408.24 ns |
| eval_complex/mixed_ops_funcs | 305.97 ns | 442.22 ns | 630.98 ns |
| eval_complex/nested_parens | 396.01 ns | 843.32 ns | 760.61 ns |
| eval_complex/trig_chain | 364.30 ns | 424.70 ns | 796.10 ns |
| eval_complex/long_addition | 362.76 ns | 413.27 ns | 496.45 ns |
| eval_scientific/sci_add | 129.09 ns | 83.29 ns | 180.91 ns |
| eval_scientific/sci_mul | 83.25 ns | 75.34 ns | 96.41 ns |
| tokenizer/simple | 122.64 ns | 102.74 ns | 112.25 ns |
| tokenizer/complex | 390.86 ns | 455.29 ns | 358.27 ns |
| unit_conversion/km_to_miles | 156.49 ns | 104.35 ns | 128.51 ns |
| unit_conversion/celsius_to_fahrenheit | 188.08 ns | 175.91 ns | 210.70 ns |
| unit_conversion/bytes_to_gb | 149.36 ns | 129.60 ns | 223.99 ns |
| unit_conversion/bytes_to_gib | 143.14 ns | 130.50 ns | 227.58 ns |
| unit_conversion/gb_to_gib_cross | 106.80 ns | 102.61 ns | 166.71 ns |
| unit_conversion/same_unit_identity | 105.63 ns | 102.35 ns | 166.75 ns |
