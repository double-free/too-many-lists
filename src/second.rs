// To learn about:
// Advanced Option use
// Generics
// Lifetimes
// Iterators

// Take aways:
//   1) Use Option<T>::take() to replace the primitive mem::replace
//   2) template syntax is `impl<T> Class<T> {...}`
//   3) Rust can infer type of container by the first inserting element
//      so we can create a list of i32 by List::new() without specifying "i32"
//   4) use as_ref() and as_mut() to borrow an Option's content
//   5) closure argument uses pattern match

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

// node is a struct of value + pointer to next
struct Node<T> {
    value: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        return List { head: None };
    }

    // a stack
    pub fn push(&mut self, elem: T) {
        let node = Box::new(Node {
            value: elem,
            next: self.head.take(),
        });

        self.head = Some(node);
    }

    pub fn pop(&mut self) -> Option<T> {
        // match option { None => None, Some(x) => Some(y) }
        // syntax sugar:
        // if None, return None
        // if Some(x), the function takes x and returns y, and wrap result with Some
        return self.head.take().map(|node| {
            self.head = node.next;
            return node.value;
        });
    }

    pub fn peek(&self) -> Option<&T> {
        return self.head.as_ref().map(|node| {
            return &node.value;
        });
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        return self.head.as_mut().map(|node| {
            return &mut node.value;
        });
    }
}

// we have to implement Drop in an iterative manner
// to avoid recursive call blowing the stack
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        // `while let` == "do this thing until this pattern doesn't match"
        while let Some(mut node) = cur_link {
            cur_link = node.next.take();
        }
    }
}

// unit tests

// only compile when running tests (cargo test), to avoid unused warning
#[cfg(test)]
mod first_list_tests {
    use super::List;

    #[test]
    fn push_and_pop() {
        // type can be infered.
        // let mut list = List::<i32>::new();
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

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        // list.peek_mut().map(|&mut value| {
        //     value = 42
        // });

        // above code does not work, because:
        // the closure argument is a "pattern match",
        // list.peek_mut() gives us a &mut T,
        // in above code, "value" is matched to "T"
        // which is not a mut ref, but a copied integer

        list.peek_mut().map(|value| {
            *value = 42;
        });

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }
}
