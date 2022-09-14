// "A Bad but Safe Doubly-Linked Deque"
// To learn about:
//   1) Become familiar with interior mutability
//   2) Safe does not mean correct

// Take-aways:
//   1. RefCell and Cell are shareable mutable containers, unlike &mut.
//   2. RefCell type checks borrow rules dynamically, which means it has runtime overhead
//   3. Because value in Rc<T> can only be borrowed by &, not &mut, it's very common to use
//      Rc<RefCell<T>> to reintroduce mutability
//   4. Only two operation matters in RefCell, borrow() and borrow_mut()
//   5. RefCell<T> is only for single-threaded scenarios, use Mutex<T> for multi-threaded situation

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

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let new_head = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(Rc::clone(&new_head));
                new_head.borrow_mut().next = Some(old_head);
                self.head = Some(new_head);
            }
            None => {
                self.head = Some(Rc::clone(&new_head));
                self.tail = Some(new_head);
            }
        }
    }
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            value: elem,
            prev: None,
            next: None,
        }))
    }
}
