// "A Production Unsafe Deque", To learn about:
//   1) How to write a standard lib level container
//   2) Variance and Subtyping
//   3) Phantom Data

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    // We semantically store values of T by-value.
    _boo: std::marker::PhantomData<T>,
}

// use NonNull to enable subtyping
type Link<T> = Option<std::ptr::NonNull<Node<T>>>;

struct Node<T> {
    prev: Link<T>,
    next: Link<T>,
    elem: T,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _boo: std::marker::PhantomData,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            // new_unchecked returns a U instead of Option<U>
            let new_head = std::ptr::NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                prev: None,
                next: None,
                elem: elem,
            })));

            match self.front {
                Some(old_head) => {
                    // Put the new front before the old one
                    (*old_head.as_ptr()).prev = Some(new_head);
                    (*new_head.as_ptr()).next = Some(old_head);
                }
                None => {
                    // If there's no front, then we're the empty list and need
                    // to set the back too. Also here's some integrity checks
                    // for testing, in case we mess up.
                    debug_assert!(self.back.is_none());
                    debug_assert!(self.front.is_none());
                    debug_assert!(self.len == 0);
                    self.back = Some(new_head);
                }
            }
            self.front = Some(new_head);
        }

        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            // Only have to do stuff if there is a front node to pop.
            // Note that we don't need to mess around with `take` anymore
            // because everything is Copy and there are no dtors that will
            // run if we mess up... right? :) Riiiight? :)))
            self.front.map(|node| {
                let boxed_node = Box::from_raw(node.as_ptr());
                let result = boxed_node.elem;

                self.front = boxed_node.next;

                match self.front {
                    Some(new_head) => {
                        // Cleanup its reference to the removed node
                        (*new_head.as_ptr()).prev = None;
                    }
                    None => {
                        // If the front is now null, then this list is now empty!
                        debug_assert!(self.len == 1);
                        self.back = None;
                    }
                }

                self.len -= 1;

                return result;
            })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_basic_front() {
        let mut list = LinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }
}
