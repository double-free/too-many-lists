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

The soundness of subtyping is based on the idea that it's ok to forget unnecessary details. But with references, there's always someone that remembers those details: the value being referenced. The problem with making `&mut T` covariant over `T` is that __it gives us the power to modify the original value when we don't remember all of its constraints__, which may result in meowing dogs:

```Rust
fn evil_feeder(pet: &mut Animal) {
    let spike: Dog = ...;

    // `pet` is an Animal, and Dog is a subtype of Animal,
    // so this should be fine, right..?
    *pet = spike;
}

fn main() {
    let mut mr_snuggles: Cat = ...;
    evil_feeder(&mut mr_snuggles);  // Replaces mr_snuggles with a Dog
    mr_snuggles.meow();             // OH NO, MEOWING DOG!
}
```

A simple rule is that __subtyping is not safe for mutable reference__. Mutable reference here means anything like `&mut T`, like `Cell<T>`, `*mut T`.

For a generic container type (e.g., the list we are creating), because of Rust's ownership system, `Container<T>` is semantically equivalent to T, and that means it's safe for it to be covariant! But our previous implementation is not covariant because we use `*mut T`. Instead, we can use `NonNull<T>`.


### Phantom Data

__Zero-sized__ type used to mark things that “act like” they own a T.

Perhaps the most common use case for PhantomData is a struct that has an unused lifetime parameter, typically as part of some unsafe code. For example, here is a struct `Slice` that has two pointers of type `*const T`, presumably pointing into an array somewhere:

```rust
struct Slice<'a, T> {
    start: *const T,
    end: *const T,
}
```

The intention is that the underlying data is only valid for the lifetime `'a`, so Slice should not outlive `'a`. However, this intent is not expressed in the code, since there are no uses of the lifetime `'a` and hence it is not clear what data it applies to. We can correct this by __telling the compiler__ to act as if the Slice struct contained a reference `&'a T`:

```rust
use std::marker::PhantomData;

struct Slice<'a, T: 'a> {
    start: *const T,
    end: *const T,
    phantom: PhantomData<&'a T>,
}
```

It is recommended by the author that __any time you do use raw pointers you should always add PhantomData__ to make it clear to compiler and others what you think you are doing.

### Exception Safety

Exception (panic, unwinding) is an __implicit early return__. Every single function immediately returns while panic. Why do we need to care about this, since the program is about to die?

The reason is that code can keep running after a panic:

- Destructors run when a function returns
- Exceptions can be caught

So, we need to make sure our unsafe collections are always in __coherent state__ whenever a panic could occur.
