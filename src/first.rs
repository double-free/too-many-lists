use std::mem;

pub struct List {
    head: Link,
}

// node is a struct of value + pointer to next
struct Node {
    value: i32,
    next: Link,
}

// we can substitute it with Option<Node>
// next item can either be empty, or a node
enum Link {
    Empty,
    More(Box<Node>),
}

impl List {
    pub fn new() -> Self {
        return List { head: Link::Empty };
    }

    // a stack
    pub fn push(&mut self, elem: i32) {
        let node = Box::new(Node {
            value: elem,
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        let head = mem::replace(&mut self.head, Link::Empty);
        match head {
            Link::Empty => {
                return None;
            }
            Link::More(node) => {
                self.head = node.next;
                return Some(node.value);
            }
        }
    }
}

// we have to implement Drop in an iterative manner
// to avoid recursive call blowing the stack
impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        loop {
            match cur_link {
                Link::Empty => {
                    // all nodes dropped
                    break;
                }

                Link::More(mut node) => {
                    // it does two things:
                    //   1) remove ownership to the next node by replacing next with Empty
                    //   2) assign next node to current node
                    cur_link = mem::replace(&mut node.next, Link::Empty);
                }
            }
        }

        // more elegant:
        // `while let` == "do this thing until this pattern doesn't match"
        // while let Link::More(mut node) = cur_link {
        //     cur_link = mem::replace(&mut node.next, Link::Empty);
        // }
    }
}

// unit tests

// only compile when running tests (cargo test), to avoid unused warning
#[cfg(test)]
mod first_list_tests {
    use super::List;

    #[test]
    fn push_and_pop() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }
}
