# abs_mm

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/ljsnogard/abs_mm)

Abstract Memory Management. 

Mod `mem_alloc` provides trait `TrMalloc` as an alternatives to unstable `Allocator` trait in core for memory allocation.  
Mod `res_man` provides a serie of traits abstracting smart pointers like `core::sync::Arc` and `core::boxed::Box`.  
Mod `as_pinned` provides trait `TrAsPinned` and `TrAsPinnedMut` for `Pin` pointers.  
