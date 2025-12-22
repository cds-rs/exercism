# Borrowing and Move Semantics in Xorcism

This document explains the ownership, borrowing, and move semantics in `lib.rs`
using ASCII diagrams similar to Rust compiler output.

---

## 1. Struct Lifetime: Xorcism borrows its key

```rust
pub struct Xorcism<'a> {
    key: &'a [u8],   // <-- borrows data with lifetime 'a
    pos: usize,      // <-- owned, no lifetime needed
}
```

```
    Owner                     Xorcism<'a>
    +-------------+           +-------------+
    | key_data    |<----------| key: &'a    |  (borrow)
    | [1, 2, 3]   |           | pos: 0      |
    +-------------+           +-------------+
         ^
         |
         +-- Xorcism can't outlive this
```

**What the compiler enforces:**
```
let key = vec![1, 2, 3];
let xs = Xorcism::new(&key);
drop(key);
     ^^^^ cannot move out of `key` because it is borrowed
xs.munge_in_place(&mut data);
-- borrow later used here
```

---

## 2. new() - Lifetime flows from input to output

```rust
pub fn new<Key: AsRef<[u8]> + ?Sized>(key: &'a Key) -> Xorcism<'a>
//                                        ^^              ^^
//                                        |               |
//                                        +-------+-------+
//                                                |
//                                      same lifetime 'a
```

```
    Caller's scope                     new() returns
    +------------------+               +------------------+
    | key: &'a [u8]    | ------------> | Xorcism<'a>      |
    | (lives for 'a)   |   as_ref()    | key: &'a [u8]    |
    +------------------+               +------------------+
                                              |
                                              v
                              tied to caller's key lifetime
```

---

## 3. munge_in_place() - Mutable borrow of self and data

```rust
pub fn munge_in_place(&mut self, data: &mut [u8])
//                    ^^^^^^^^^        ^^^^^^^^^^
//                    |                |
//                    |                +-- mutable borrow of data
//                    +-- mutable borrow of self
```

```
    Timeline
    --------

    let mut xs = Xorcism::new(&key);
        ------ xs is valid here

    let mut data = [1, 2, 3];
        -------- data is valid here

    xs.munge_in_place(&mut data);
    ^^                ^^^^^^^^^
    |                 |
    |                 +-- data mutably borrowed (can't use data until call ends)
    +-- xs mutably borrowed (can't use xs until call ends)

    // After the call, both borrows end
    xs.munge_in_place(&mut data);  // OK to use again
```

---

## 4. munge() - The spicy one: returned iterator borrows self

```rust
pub fn munge<'b, Data>(&'b mut self, data: Data) -> impl Iterator<Item = u8> + 'b
//           ^^        ^^^^^^^^^^^                                           ^^^^
//           |         |                                                     |
//           |         +-- self borrowed for 'b                              |
//           |                                                               |
//           +---------------------------+-----------------------------------+
//                                       |
//                            iterator lives for 'b
//                            (holds onto the borrow)
```

```
    Timeline
    --------

    let mut xs = Xorcism::new(&key);

    let iter = xs.munge(data);
               ^^
               |
               +-- mutable borrow of xs starts here
                   |
                   v
    +----------------------------------+
    | iter holds &mut xs               |
    | xs cannot be used while iter     |
    | exists                           |
    +----------------------------------+

    xs.munge(more_data);
    ^^
    |
    +-- ERROR: cannot borrow `xs` as mutable more than once
        first mutable borrow: iter
        second mutable borrow: here

    drop(iter);  // <-- borrow ends here

    xs.munge(more_data);  // OK now
```

**Inside the closure:**
```rust
data.into_iter().map(move |byte| {
//                   ^^^^
//                   |
//                   +-- moves `self` (which is &'b mut Xorcism)
//                       into the closure
    self.pos += 1;
//  ^^^^ closure owns the mutable reference, can mutate
})
```

```
    Before move              After move
    +------------+           +-------------------+
    | self: &mut |           | closure captures: |
    | Xorcism    | --------> | self: &mut        |
    +------------+   move    | Xorcism           |
         |                   +-------------------+
         v                          |
    (no longer                      v
     accessible               (closure owns it,
     in munge())               updates pos)
```

---

## 5. reader() and writer() - Self is consumed (moved)

```rust
pub fn reader<R: Read>(self, reader: R) -> XorReader<'a, R>
//                     ^^^^
//                     |
//                     +-- takes ownership of self (not &self or &mut self)
```

```
    Timeline
    --------

    let xs = Xorcism::new(&key);
        -- xs created, owns Xorcism

    let rd = xs.reader(source);
             ^^
             |
             +-- xs MOVED into reader()
                 |
                 v
    +---------------------------+
    |  xs is now invalid        |
    |  Xorcism lives inside rd  |
    +---------------------------+

    xs.munge_in_place(&mut data);
    ^^
    |
    +-- ERROR: borrow of moved value: `xs`
        value moved here: xs.reader(source)
```

```
    Before reader()              After reader()
    +-------------+              +--------------------+
    | xs:         |              | rd: XorReader      |
    | Xorcism     | -----------> |   xor: Xorcism <---+-- xs moved here
    +-------------+    move      |   reader: R        |
         |                       +--------------------+
         v
    (xs is gone,
     can't use it)
```

---

## 6. XorReader/XorWriter - Nested borrows in read()/write()

```rust
fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
//      ^^^^^^^^^  ^^^^^^^^^^^^
//      |          |
//      |          +-- caller's buffer, borrowed mutably
//      +-- XorReader borrowed mutably
```

```
    +------------------+
    | XorReader        |
    |   +------------+ |
    |   | xor:       | |  <-- also mutably accessible via &mut self
    |   | Xorcism    | |
    |   +------------+ |
    |   +------------+ |
    |   | reader: R  | |  <-- self.reader.read(buf) borrows this
    |   +------------+ |
    +------------------+
            |
            v
    &mut self gives access to both fields
    but Rust ensures no conflicting borrows
```

---

## 7. Stack buffer in write() - No ownership transfer

```rust
fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let mut stack_buffer = [0u8; 1024];
//      ^^^^^^^^^^^^^^^^^
//      |
//      +-- owned by this stack frame

    stack_buffer[..len].copy_from_slice(&buf[..len]);
//                                      ^^^^^^^^^^
//                                      |
//                                      +-- immutable borrow of buf (just reading)

    self.xor.munge_in_place(&mut stack_buffer[..len]);
//                          ^^^^^^^^^^^^^^^^^^^^^^^^^
//                          |
//                          +-- mutable borrow of stack_buffer

    self.writer.write(&stack_buffer[..len])
//                    ^^^^^^^^^^^^^^^^^^^^^
//                    |
//                    +-- immutable borrow (munge is done, we're just reading)
}
// stack_buffer dropped here (stack frame ends)
```

---

## Summary: The Three Types of "self"

```
+------------------+-------------------+--------------------------------+
| Signature        | What happens      | After the call                 |
+------------------+-------------------+--------------------------------+
| &self            | immutable borrow  | self still usable              |
| &mut self        | mutable borrow    | self still usable              |
| self             | MOVE (ownership)  | self is GONE, can't use it     |
+------------------+-------------------+--------------------------------+
```

```
munge_in_place(&mut self, ...)  -->  xs still valid after call
munge(&mut self, ...)           -->  xs borrowed until iterator dropped
reader(self, ...)               -->  xs moved, gone forever
```

---

## Visualizing Lifetimes

```
'a: |===========================================|  (key's lifetime)

    |     Xorcism<'a>     |                        (must fit inside 'a)

'b:       |=====|                                  (munge's borrow)

          | iter |                                 (iterator lives for 'b)
```

The Xorcism can't outlive the key ('a).
The iterator can't outlive the munge borrow ('b).
'b can't exceed the Xorcism's lifetime.
Lifetimes nest. Borrows nest. The compiler enforces all of it.
