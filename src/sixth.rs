// "A Production Unsafe Deque", To learn about:
//   1) How to write a standard lib level container
//   2) Variance and Subtyping
//   3) Phantom Data
//   4) Send and Sync traits

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

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let new_tail = std::ptr::NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                prev: None,
                next: None,
                elem: elem,
            })));

            if let Some(old_tail) = self.back {
                (*old_tail.as_ptr()).next = Some(new_tail);
                (*new_tail.as_ptr()).prev = Some(old_tail);
            } else {
                // no existing element, the new one is both the front and back
                self.front = Some(new_tail);
            }

            self.back = Some(new_tail);
        }
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        // Only have to do stuff if there is a front node to pop.
        // Note that we don't need to mess around with `take` anymore
        // because everything is Copy and there are no dtors that will
        // run if we mess up... right? :) Riiiight? :)))
        self.front.map(|node| unsafe {
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

    pub fn pop_back(&mut self) -> Option<T> {
        self.back.map(|node_ptr| unsafe {
            let boxed_node = Box::from_raw(node_ptr.as_ptr());
            // moved value out
            let result = boxed_node.elem;

            self.back = boxed_node.prev;
            if let Some(new_tail) = self.back {
                (*new_tail.as_ptr()).next = None;
            } else {
                // last element is popped
                self.front = None;
            }

            self.len -= 1;
            return result;
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn front(&self) -> Option<&T> {
        self.front.map(|node| unsafe { &node.as_ref().elem })
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.front
            .map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn back(&self) -> Option<&T> {
        self.back.map(|node| unsafe { &node.as_ref().elem })
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.back.map(|mut node| unsafe { &mut node.as_mut().elem })
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

// const iterator
pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        return self.front.map(|node| unsafe {
            self.front = (*node.as_ptr()).next;
            &node.as_ref().elem
        });
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        return self.back.map(|node| unsafe {
            self.back = (*node.as_ptr()).prev;
            &node.as_ref().elem
        });
    }
}

impl<T> LinkedList<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _boo: std::marker::PhantomData,
        }
    }
}

// mutable iterator
pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        return self.front.map(|mut node| unsafe {
            self.len -= 1;
            self.front = node.as_ref().next;
            &mut node.as_mut().elem
        });
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        return self.back.map(|mut node| unsafe {
            self.back = node.as_ref().prev;
            &mut node.as_mut().elem
        });
    }
}

impl<T> LinkedList<T> {
    pub fn iter_mut(&self) -> IterMut<T> {
        IterMut {
            front: self.front,
            back: self.back,
            len: self.len,
            _boo: std::marker::PhantomData,
        }
    }
}

// into iterator
pub struct IntoIter<T>(LinkedList<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len, Some(self.0.len))
    }
}

impl<T> LinkedList<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { 0: self }
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

//////////////////////
// other interfaces //
//////////////////////

impl<T: std::hash::Hash> std::hash::Hash for LinkedList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for item in self.iter() {
            item.hash(state);
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut new_list = Self::new();
        for item in self.iter() {
            new_list.push_back(item.clone());
        }
        new_list
    }
}

impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

// if T implements PartialEq, so does LinkedList<T>
impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}
impl<T: Eq> Eq for LinkedList<T> {}

impl<T: PartialOrd> PartialOrd for LinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}
impl<T: Ord> Ord for LinkedList<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

// Sync and Send are "unsafe" traits
unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

unsafe impl<'a, T: Send> Send for Iter<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}

#[cfg(test)]
mod test {
    use super::LinkedList;

    fn generate_test() -> LinkedList<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    fn list_from<T: Clone>(v: &[T]) -> LinkedList<T> {
        v.iter().map(|x| (*x).clone()).collect()
    }

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

    #[test]
    fn test_basic() {
        let mut m = LinkedList::new();
        assert_eq!(m.pop_front(), None);
        assert_eq!(m.pop_back(), None);
        assert_eq!(m.pop_front(), None);
        m.push_front(1);
        assert_eq!(m.pop_front(), Some(1));
        m.push_back(2);
        m.push_back(3);
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_front(), Some(2));
        assert_eq!(m.pop_front(), Some(3));
        assert_eq!(m.len(), 0);
        assert_eq!(m.pop_front(), None);
        m.push_back(1);
        m.push_back(3);
        m.push_back(5);
        m.push_back(7);
        assert_eq!(m.pop_front(), Some(1));

        let mut n = LinkedList::new();
        n.push_front(2);
        n.push_front(3);
        {
            assert_eq!(n.front().unwrap(), &3);
            let x = n.front_mut().unwrap();
            assert_eq!(*x, 3);
            *x = 0;
        }
        {
            assert_eq!(n.back().unwrap(), &2);
            let y = n.back_mut().unwrap();
            assert_eq!(*y, 2);
            *y = 1;
        }
        assert_eq!(n.pop_front(), Some(0));
        assert_eq!(n.pop_front(), Some(1));
    }

    #[test]
    fn test_iterator() {
        let m = generate_test();
        for (i, elt) in m.iter().enumerate() {
            assert_eq!(i as i32, *elt);
        }
        let mut n = LinkedList::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_double_end() {
        let mut n = LinkedList::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(it.next().unwrap(), &6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.next_back().unwrap(), &4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next_back().unwrap(), &5);
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_rev_iter() {
        let m = generate_test();
        for (i, elt) in m.iter().rev().enumerate() {
            assert_eq!(6 - i as i32, *elt);
        }
        let mut n = LinkedList::new();
        assert_eq!(n.iter().rev().next(), None);
        n.push_front(4);
        let mut it = n.iter().rev();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_mut_iter() {
        let m = generate_test();
        let mut len = m.len();
        for (i, elt) in m.iter_mut().enumerate() {
            assert_eq!(i as i32, *elt);
            len -= 1;
        }
        assert_eq!(len, 0);
        let mut n = LinkedList::new();
        assert!(n.iter_mut().next().is_none());
        n.push_front(4);
        n.push_back(5);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iterator_mut_double_end() {
        let mut n = LinkedList::new();
        assert!(n.iter_mut().next_back().is_none());
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(*it.next().unwrap(), 6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(*it.next_back().unwrap(), 4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(*it.next_back().unwrap(), 5);
        assert!(it.next_back().is_none());
        assert!(it.next().is_none());
    }

    #[test]
    fn test_eq() {
        let mut n: LinkedList<u8> = list_from(&[]);
        let mut m = list_from(&[]);
        assert!(n == m);
        n.push_front(1);
        assert!(n != m);
        m.push_back(1);
        assert!(n == m);

        let n = list_from(&[2, 3, 4]);
        let m = list_from(&[1, 2, 3]);
        assert!(n != m);
    }

    #[test]
    fn test_ord() {
        let n = list_from(&[]);
        let m = list_from(&[1, 2, 3]);
        assert!(n < m);
        assert!(m > n);
        assert!(n <= n);
        assert!(n >= n);
    }

    #[test]
    fn test_ord_nan() {
        let nan = 0.0f64 / 0.0;
        let n = list_from(&[nan]);
        let m = list_from(&[nan]);
        assert!(!(n < m));
        assert!(!(n > m));
        assert!(!(n <= m));
        assert!(!(n >= m));

        let n = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(n < one));
        assert!(!(n > one));
        assert!(!(n <= one));
        assert!(!(n >= one));

        let u = list_from(&[1.0f64, 2.0, nan]);
        let v = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(u < v));
        assert!(!(u > v));
        assert!(!(u <= v));
        assert!(!(u >= v));

        let s = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(s < t));
        assert!(s > one);
        assert!(!(s <= one));
        assert!(s >= one);
    }

    #[test]
    fn test_debug() {
        let list: LinkedList<i32> = (0..10).collect();
        assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let list: LinkedList<&str> = vec!["just", "one", "test", "more"]
            .iter()
            .copied()
            .collect();
        assert_eq!(format!("{:?}", list), r#"["just", "one", "test", "more"]"#);
    }

    #[test]
    fn test_hashmap() {
        // Check that HashMap works with this as a key

        let list1: LinkedList<i32> = (0..10).collect();
        let list2: LinkedList<i32> = (1..11).collect();
        let mut map = std::collections::HashMap::new();

        assert_eq!(map.insert(list1.clone(), "list1"), None);
        assert_eq!(map.insert(list2.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&list1), Some(&"list1"));
        assert_eq!(map.get(&list2), Some(&"list2"));

        assert_eq!(map.remove(&list1), Some("list1"));
        assert_eq!(map.remove(&list2), Some("list2"));

        assert!(map.is_empty());
    }
}

// test Sync and Send traits
#[allow(dead_code)]
fn assert_properties() {
    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    is_send::<LinkedList<i32>>();
    is_sync::<LinkedList<i32>>();

    is_send::<IntoIter<i32>>();
    is_sync::<IntoIter<i32>>();

    is_send::<Iter<i32>>();
    is_sync::<Iter<i32>>();

    is_send::<IterMut<i32>>();
    is_sync::<IterMut<i32>>();

    // is_send::<Cursor<i32>>();
    // is_sync::<Cursor<i32>>();

    fn linked_list_covariant<'a, T>(x: LinkedList<&'static T>) -> LinkedList<&'a T> {
        x
    }
    fn iter_covariant<'i, 'a, T>(x: Iter<'i, &'static T>) -> Iter<'i, &'a T> {
        x
    }
    fn into_iter_covariant<'a, T>(x: IntoIter<&'static T>) -> IntoIter<&'a T> {
        x
    }
}
