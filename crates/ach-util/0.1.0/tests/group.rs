use ach_util::{AtomicMemoryGroup, MemoryGroup, MemoryState};
use core::sync::atomic::Ordering::Relaxed;

#[test]
fn test() {
    assert_eq!(AtomicMemoryGroup::ZERO.load(Relaxed), MemoryGroup::new());
    let mut group = MemoryGroup::new();
    assert_eq!(group.group(), 0);
    assert!(group.state().is_uninitialized());

    group.set_group(1);
    assert_eq!(group.group(), 1);
    group.set_group(MemoryGroup::max_group());
    assert_eq!(group.group(), 0);
    group.set_state(MemoryState::Initialized);
    assert!(group.state().is_initialized());

    group.set_group(MemoryGroup::max_group()-1);
    group.set_state(MemoryState::Erasing);
    group = group.next();
    assert_eq!(group.group(), 0);
    assert!(group.state().is_uninitialized());
}
