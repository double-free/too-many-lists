// "Persist List", To learn about:
//   reference counting in Rust: Rc and Arc

// Takeaways:
//   1. Option<T>::map argument function must return a valid value (U), while
//      Option<T>::and_then argument function can return nullable Option<U>
//   2. It is immutable, because the prepend(), tail() both return a new List
//   3. Like the mutable list (Box instead of Rc), we need to fix the "recursive destructor"
//   4. Rc::try_unwrap can TAKE the value of the RC pointer if there is only 1 strong ref.

use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn prepend(&self, elem: T) -> List<T> {
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

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(shared_node) = head {
            // try do drop the value
            match Rc::try_unwrap(shared_node) {
                Ok(node) => {
                    head = node.next;
                }
                Err(_) => break,
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }
}
