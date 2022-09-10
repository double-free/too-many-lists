// "A Bad but Safe Doubly-Linked Deque"
// To learn about:
//   1) Become familiar with interior mutability
//   2) Safe does not mean correct

// Take-aways:
//   1. RefCell type checks borrow rules dynamically

use std::cell::RefCell;
use std::rc::Rc;

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

struct Node<T> {
    value: T,
    next: Link<T>,
    prev: Link<T>,
}
