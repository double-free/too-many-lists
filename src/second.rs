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
//   6) use associate type instead of genrics for a trait
//   7) "lifetime": it's basically an extra implicit argument.
//      usually it can be infered by the compiler from input. It guarantees
//      the input ref MUST live at least as long as the output ref.
//   8) use as_deref() to convert Option<T> to Option<&T>
//   9) iter_mut will take the ownership of element
//   10) Option<T>::map consumes the option itself.

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

// take the list itself
pub struct IntoIter<T>(List<T>);
// const ref
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

// iterators
impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        return IntoIter(self);
    }
    pub fn iter(&self) -> Iter<T> {
        // we want an Option<&Node<T>>, but we have an Option<Node<T>>
        // as_deref() converts from Option<T> to Option<&T>
        return Iter {
            next: self.head.as_deref(),
        };
    }
    pub fn iter_mut(&mut self) -> IterMut<T> {
        return IterMut {
            next: self.head.as_deref_mut(),
        };
    }
}

impl<T> Iterator for IntoIter<T> {
    // See usage of "associate type" in traits here:
    //   https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
    // Using associate types (instead of genrics Iterator<T>) restricts that
    // we have only 1 implementation for a trait on a type
    // (we can't have Iterator<String> and Iterator<i32> at the same time)
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        return self.0.pop();
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    // Item is a const ref of T
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        // the "self.next" (an Option<&Node<T>>) is consumed and updated
        self.next.map(|node| {
            // node.next is an Option<Node<T>>, but we want to return Option<&Node<T>>
            // still, we need as_deref
            self.next = node.next.as_deref();
            return &node.value;
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    // Item is a mutable ref of T
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        // "take" ownership of "self.next"
        // which is an iterator "Option<&'a mut Node<T>>,"
        // not the node itself
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            return &mut node.value;
        })
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

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));

        let mut iter1 = list.iter();
        assert_eq!(iter1.next(), Some(&3));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));

        // modify
        for value in list.iter_mut() {
            *value *= 2;
        }

        let mut iter1 = list.iter_mut();
        assert_eq!(iter1.next(), Some(&mut 6));
    }
}
