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
//   6. Rc<T> can't release if it holds strong ref to each other,
//      so we need to manually pop elements in Drop
//   7. When we borrow from a RefCell, we get a Ref<T> type (instead of &T), which is a reference with lifetime
//      This is how it implements dynamic borrow checking.

use std::cell::{Ref, RefCell};
use std::rc::Rc;

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

struct Node<T> {
    elem: T,
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

    pub fn pop_front(&mut self) -> Option<T> {
        return self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                None => {
                    // reset tail
                    self.tail = None;
                }
                Some(new_head) => {
                    new_head.borrow_mut().prev = None;
                    self.head = Some(new_head);
                }
            }
            return Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem;
        });
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |borrowed| &borrowed.elem))
    }
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem: elem,
            prev: None,
            next: None,
        }))
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(&*list.peek_front().unwrap(), &3);
    }
}
