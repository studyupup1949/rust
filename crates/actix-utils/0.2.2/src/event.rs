use std::cell::Cell;
use std::rc::Rc;

use futures::task::AtomicTask;

pub struct Event {
    count: Cell<usize>,
    capacity: usize,
    task: AtomicTask,
}

pub struct EventProvider {
    (),
}

struct EventInner {
    count: Cell<usize>,
    capacity: usize,
    task: AtomicTask,
}
