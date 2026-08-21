# abs_mm

Abstract Memory Management. 

Currently (Oct 2024) only for rustc nightly.

Mod `mem_alloc` provides traits for memory allocation.  
Enabling `support-std` feature will provide `StdGlobalAlloc` which implements `TrMalloc`.  

Mod `res_man` provides traits describing smart pointers like `std::sync::Arc` and `std::boxed::Box`.  
