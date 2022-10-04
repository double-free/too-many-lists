# Rust "Too Many Lists" Implementation

Follows https://rust-unofficial.github.io/too-many-lists to learn Rust.

Highlights key points of implemention at top of the source file.


## Things to learn

This tutorial provides an insight of how to write memory-safe program, it's much more than just a Rust tutorial.

### Avoid recursive destructor

Let's consider a simple list:

```text
list -> A -> B -> C
```

When list gets dropped, it will try to drop A, which will try to drop B, which will try to drop C. This is __recursive__ code that may blow up the stack.

To avoid this, we need to implement an iterative version of `Drop`.

```rust
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        // `while let` == "do this thing until this pattern doesn't match"
        while let Some(mut node) = cur_link {
            cur_link = node.next.take();
        }
    }
}
```

### Stacked-borrow rule

Rust can keep the memory safety even when an object is re-borrowed mutably. This is achieved by "stacked-borrow rule".

```rust
let mut data = 10;
let ref1 = &mut data;
let ref2 = &mut *ref1;

*ref2 += 2;
*ref1 += 1;

println!("{}", data);
```

In above example, We reborrow the pointer, use the new pointer for a while, and then stop using it before using the older pointer again.

This is how we can have reborrows and still have aliasing information: all of our reborrows clearly nest, so we can consider only __one__ of them "live" at any given time.

But if we swap `ref1` and `ref2`, their lifetimes overlapped, and the stacked-borrow rule is broken:

```rust
let mut data = 10;
let ref1 = &mut data;
let ref2 = &mut *ref1;

// ORDER SWAPPED!
*ref1 += 1;
*ref2 += 2;

println!("{}", data);
```

Rust will give us a compiler error.


### Essentials of lifetime

Every reference has two template parameters: the lifetime and the underlying type. The lifetime is sometimes hidden from users when it can be inferred.

Lifetimes are just regions of code, and regions can be partially ordered with thecontains (outlives) relationship. Subtyping on lifetimes is in terms of that relationship:

> if 'big: 'small ("big contains small" or "big outlives small"), then 'big is a subtype of 'small.

Put another way, if someone wants a reference that lives for 'small, usually what they actually mean is that they want a reference that lives for at least 'small. They don't actually care if the lifetimes match exactly. So it should be ok for us to forget that something lives for 'big and only remember that it lives for 'small.

### Variance and Subtyping

Subtyping is a relationship between types that allows statically typed languages to be a bit more flexible and permissive. __Lifetime is an example of subtyping in Rust__: if `'big: 'small` ("big contains small" or "big outlives small"), then `'big` is a subtype of `'small`.

Variance is a property that type constructors have with respect to their arguments. e.g., & and &mut are type constructors that take two inputs: a lifetime, and a type to point to.

A type constructor F's variance is how the subtyping of its inputs affects the subtyping of its outputs.

- F is covariant if F<Sub> is a subtype of F<Super> (subtyping "passes through")
- F is contravariant if F<Super> is a subtype of F<Sub> (subtyping is "inverted")
- F is invariant otherwise (no subtyping relationship exists)

| type | `'a` | T | U |
| :-: | :-: | :-: | :-: |
| `&'a T`| covariant | covariant | |
| `&'a mut T`| covariant | invariant | |
| `Box<T>` |  | covariant | |
| `Vec<T>` |  | covariant | |
| `UnsafeCell<T>` | | invariant | |
| `Cell<T>` | | invariant | |
| `fn(T) -> U` | | contravariant | covariant |
| `*const T` | | covariant | |
| `*mut T` | | invariant | |

Apparently, we want to keep `covariant` if safety is guaranteed. If a function accepts a `Base` instance, then we expect it to accept `Derived` instance.

A simple rule is that __subtyping is not safe for mutable reference__. Mutable reference here means anything like `&mut T`, like `Cell<T>`, `*mut T`.

For a generic container type (e.g., the list we are creating), because of Rust's ownership system, `Container<T>` is semantically equivalent to T, and that means it's safe for it to be covariant! But our previous implementation is not covariant because we use `*mut T`. Instead, we can use `NonNull<T>`.


### Phantom Data
