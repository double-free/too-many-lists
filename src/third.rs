// To learn about:
//   reference counting in Rust: Rc and Arc

// Takeaways:
//   1. Option<T>::map argument function must return a valid value (U), while
//      Option<T>::and_then argument function can return another Option<U>

use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn prepend(&mut self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                value: elem,
                next: self.head.clone(),
            })),
        }
    }

    // tail is not a good name, it returns the list without the head
    pub fn tail(&self) -> List<T> {
        // and_then: if node is None, return None,
        //           if node is valid, return the function's result (unlike map, can be None)
        List {
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }
}
