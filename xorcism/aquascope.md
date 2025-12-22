# Aquascope-Style Permission Analysis

This document shows Read (R), Write (W), and Own (O) permissions at each
program point, similar to how [Aquascope](https://cognitive-engineering-lab.github.io/aquascope/) visualizes them.

Legend:
- `R` = can read
- `W` = can write/mutate
- `O` = can move or drop (ownership)
- `-` = permission revoked
- `x` = path no longer valid

---

## 1. Creating a Xorcism (new)

```rust
L1:  let key = vec![1, 2, 3, 4, 5];
L2:  let xs = Xorcism::new(&key);
L3:  // use xs and key here
L4:  drop(xs);
L5:  drop(key);
```

| Line | `key`   | `xs`    | Notes                                      |
|------|---------|---------|-------------------------------------------|
| L1   | R W O   | -       | key created, full permissions              |
| L2   | R - O   | R W O   | key lends to xs, loses W while borrowed    |
| L3   | R - O   | R W O   | xs holds &key, key can't be mutated        |
| L4   | R W O   | x       | xs dropped, borrow ends, key regains W     |
| L5   | x       | x       | key dropped                                |

**Why key loses W at L2:**
xs contains `&'a [u8]` pointing into key. If key could be mutated, that
reference might become invalid (e.g., vec reallocation). Rust prevents this.

---

## 2. munge_in_place - Temporary mutable borrows

```rust
L1:  let key = [1, 2, 3];
L2:  let mut xs = Xorcism::new(&key);
L3:  let mut data = [10, 20, 30];
L4:  xs.munge_in_place(&mut data);
L5:  println!("{:?}", data);
L6:  xs.munge_in_place(&mut data);
```

| Line | `key`   | `xs`    | `data`  | Notes                                |
|------|---------|---------|---------|--------------------------------------|
| L1   | R W O   | -       | -       | key created                          |
| L2   | R - O   | R W O   | -       | xs borrows key                       |
| L3   | R - O   | R W O   | R W O   | data created                         |
| L4   | R - O   | - - -   | - - -   | DURING call: both mutably borrowed   |
| L4'  | R - O   | R W O   | R W O   | AFTER call: borrows returned         |
| L5   | R - O   | R W O   | R - O   | data immutably borrowed by println   |
| L5'  | R - O   | R W O   | R W O   | after println                        |
| L6   | R - O   | - - -   | - - -   | same pattern as L4                   |

**The mutable borrow dance:**
```
xs.munge_in_place(&mut data)
^^                ^^^^^^^^^
|                 |
|                 +-- data: R W O --> - - - (during call)
|
+-- xs: R W O --> - - - (during call, needs &mut self)
```

---

## 3. munge - Iterator holds the borrow

```rust
L1:  let key = [1, 2, 3];
L2:  let mut xs = Xorcism::new(&key);
L3:  let data = [10, 20, 30];
L4:  let iter = xs.munge(&data);
L5:  // iter exists, xs is locked
L6:  let result: Vec<u8> = iter.collect();
L7:  // iter consumed, xs is free
L8:  let iter2 = xs.munge(&data);
```

| Line | `key`   | `xs`    | `data`  | `iter`  | Notes                          |
|------|---------|---------|---------|---------|--------------------------------|
| L1   | R W O   | -       | -       | -       |                                |
| L2   | R - O   | R W O   | -       | -       |                                |
| L3   | R - O   | R W O   | R W O   | -       |                                |
| L4   | R - O   | - - -   | R - O   | R W O   | xs locked! iter owns &mut xs   |
| L5   | R - O   | - - -   | R - O   | R W O   | can't touch xs                 |
| L6   | R - O   | - - -   | R - O   | - - -   | iter being consumed            |
| L6'  | R - O   | R W O   | R W O   | x       | iter gone, xs unlocked         |
| L7   | R - O   | R W O   | R W O   | x       |                                |
| L8   | R - O   | - - -   | R - O   | R W O   | new iter, xs locked again      |

**What would fail at L5:**
```rust
let iter = xs.munge(&data);
xs.munge_in_place(&mut other);  // ERROR!
// ^^ cannot borrow `xs` as mutable because it is also borrowed by `iter`
```

| Path   | Permission at L5 | Why                                    |
|--------|------------------|----------------------------------------|
| `xs`   | - - -            | mutable borrow held by iter            |
| `iter` | R W O            | iter owns the &mut xs reference        |

---

## 4. reader() - Move semantics (ownership transfer)

```rust
L1:  let key = [1, 2, 3];
L2:  let xs = Xorcism::new(&key);
L3:  let source: &[u8] = &[10, 20, 30];
L4:  let reader = xs.reader(source);
L5:  // xs is GONE
L6:  // reader owns the Xorcism now
```

| Line | `key`   | `xs`    | `source` | `reader` | Notes                        |
|------|---------|---------|----------|----------|------------------------------|
| L1   | R W O   | -       | -        | -        |                              |
| L2   | R - O   | R W O   | -        | -        |                              |
| L3   | R - O   | R W O   | R W O    | -        |                              |
| L4   | R - O   | x x x   | R - O    | R W O    | xs MOVED into reader         |
| L5   | R - O   | x x x   | R - O    | R W O    | xs is not "-", it's "x" gone |
| L6   | R - O   | x x x   | R - O    | R W O    |                              |

**Move vs Borrow:**
```
Borrow:  R W O  -->  - - -  -->  R W O   (temporary, comes back)
Move:    R W O  -->  x x x                (permanent, path invalid)
```

**What would fail at L5:**
```rust
let reader = xs.reader(source);
xs.munge_in_place(&mut data);  // ERROR!
// ^^ borrow of moved value: `xs`
//    value moved here: xs.reader(source)
```

---

## 5. Closure capture in munge()

```rust
pub fn munge<'b, Data>(&'b mut self, data: Data) -> impl Iterator + 'b {
L1:      data.into_iter().map(move |byte| {
L2:          let key_byte = self.key[self.pos % self.key.len()];
L3:          self.pos += 1;
L4:          *byte.borrow() ^ key_byte
L5:      })
}
```

**Permissions INSIDE munge(), before closure:**

| Point    | `self`  | `data`  | Notes                              |
|----------|---------|---------|-----------------------------------|
| entry    | R W -   | R W O   | self is &mut (no O), data is owned |
| L1       | - - -   | - - -   | both moved into iterator chain     |

**The `move` keyword effect:**
```
Before move |byte| { ... }     After move |byte| { ... }
+------------------+           +------------------------+
| self: &'b mut    |           | closure captures:      |
| (R W -)          |  move     | self: &'b mut (R W -)  |
+------------------+  ------>  +------------------------+
       |                              |
       v                              v
  accessible in               closure owns it
  munge() scope               munge() can't use self
```

**Why `self` doesn't have O:**
`self` is `&'b mut Xorcism`, a mutable reference. References don't have
ownership (O) of what they point to. They have R and W (for &mut).

---

## 6. XorReader::read() - Nested access

```rust
fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
L1:      let n = self.reader.read(buf)?;
L2:      self.xor.munge_in_place(&mut buf[..n]);
L3:      Ok(n)
}
```

| Point | `self`  | `self.reader` | `self.xor` | `buf`   | Notes               |
|-------|---------|---------------|------------|---------|---------------------|
| entry | R W -   | R W -         | R W -      | R W -   | all mut borrowed    |
| L1    | R - -   | - - -         | R W -      | - - -   | reader + buf in use |
| L1'   | R W -   | R W -         | R W -      | R W -   | call returns        |
| L2    | R - -   | R W -         | - - -      | - - -   | xor + buf in use    |
| L2'   | R W -   | R W -         | R W -      | R W -   | call returns        |
| L3    | R W -   | R W -         | R W -      | R W -   | about to return     |

**Rust's split borrow magic:**
At L1, we borrow `self.reader` and `buf`. We don't touch `self.xor`.
At L2, we borrow `self.xor` and `buf`. We don't touch `self.reader`.

The compiler sees these are disjoint fields and allows it. This wouldn't
work if we tried to use the same field twice simultaneously.

---

## 7. XorWriter::write() - Stack buffer lifecycle

```rust
fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
L1:      let mut stack_buffer = [0u8; 1024];
L2:      let len = buf.len().min(stack_buffer.len());
L3:      stack_buffer[..len].copy_from_slice(&buf[..len]);
L4:      self.xor.munge_in_place(&mut stack_buffer[..len]);
L5:      self.writer.write(&stack_buffer[..len])
}  // L6: stack_buffer dropped
```

| Point | `buf`   | `stack_buffer` | `self.xor` | `self.writer` |
|-------|---------|----------------|------------|---------------|
| entry | R - -   | -              | R W -      | R W -         |
| L1    | R - -   | R W O          | R W -      | R W -         |
| L2    | R - -   | R W O          | R W -      | R W -         |
| L3    | R - -   | - W -          | R W -      | R W -         |
| L3'   | R - -   | R W O          | R W -      | R W -         |
| L4    | R - -   | - - -          | - - -      | R W -         |
| L4'   | R - -   | R W O          | R W -      | R W -         |
| L5    | R - -   | R - -          | R W -      | - - -         |
| L5'   | R - -   | R W O          | R W -      | R W -         |
| L6    | R - -   | x x x          | R W -      | R W -         |

**Why buf is R - - (no W, no O):**
`buf: &[u8]` is an immutable borrow. We can read but not write or move.

**Why stack_buffer gets O:**
It's a local variable. We own it. We can read, write, and drop it.

---

## Permission Rules Summary

| Operation              | Lender loses | Borrower gains | Duration    |
|------------------------|--------------|----------------|-------------|
| `&T` (shared borrow)   | W            | R              | temporary   |
| `&mut T` (mut borrow)  | R W O        | R W            | temporary   |
| move                   | R W O        | R W O          | permanent   |

| Self type    | Permissions | What it means                              |
|--------------|-------------|--------------------------------------------|
| `&self`      | R - -       | can read self                              |
| `&mut self`  | R W -       | can read and write self                    |
| `self`       | R W O       | owns self, can do anything including drop  |

---

## Seeing Permission Flow

```
let mut x = String::from("hello");  // x: R W O

let y = &x;                         // x: R - O, y: R - -
                                    //      ^        ^
                                    //      |        +-- y can read
                                    //      +-- x loses W (can't mutate while borrowed)

drop(y);                            // x: R W O (W returns)

let z = &mut x;                     // x: - - -, z: R W -
                                    //    ^^^^^       ^^^
                                    //    |           +-- z can read and write
                                    //    +-- x loses everything temporarily

drop(z);                            // x: R W O (all return)

let w = x;                          // x: x x x, w: R W O
                                    //    ^^^^^       ^^^
                                    //    |           +-- w now owns it
                                    //    +-- x is GONE (moved, not borrowable)
```
