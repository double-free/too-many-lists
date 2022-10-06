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
    front: Link<T>,
    back: Link<T>,
    elem: T,
}
