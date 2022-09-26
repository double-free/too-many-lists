// "Unsafe Singly-Linked Queue", similar to "second.rs", To learn about:
//   raw pointers and Unsafe Rust.

// Takeaways:
//   1. unsafe is manageable because of privacy. As long as no combination
//      of the APIs we expose causes bad stuff to happen, from user's perspective,
//      the module is safe!
//   2. "stacked borrow" can be safe, if we reborrow a ref, mutate it,
//      and stop using the borrowed reference before using the old ref again.
//      rust can check it in compile time
//   3. when you convert a reference into a raw pointer, it is basically a "reborrow",
//      but, unlike normal reference, rust can't check the stacked borrow for a raw pointer
//   4. because of 3, the Box can't claim unique ownership of memory because a mutable raw pointer exists
//   5. Once you start using raw pointers, try to ONLY use raw pointers.

// Version 1: using Box and raw pointer, breaks the stacked borrow rule of Box
// pub struct List<T> {
//     head: Link<T>,
//     tail: *mut Node<T>,
// }
// type Link<T> = Option<Box<Node<T>>>;

// Version 2: use raw pointer only

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}
type Link<T> = *mut Node<T>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        unsafe {
            let new_tail = Box::into_raw(Box::new(Node {
                elem: elem,
                next: std::ptr::null_mut(),
            }));

            if self.tail.is_null() {
                // only 1 element, header is the same as tail
                self.head = new_tail;
            } else {
                (*self.tail).next = new_tail;
            }

            self.tail = new_tail;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                return None;
            }

            let old_head = self.head;
            self.head = (*self.head).next;

            if self.head.is_null() {
                // no element after pop, reset tail, too.
                self.tail = std::ptr::null_mut();
            }

            return Some(Box::from_raw(old_head).elem);
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
