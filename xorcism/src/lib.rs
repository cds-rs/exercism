use std::borrow::Borrow;
#[cfg(feature = "io")]
use std::io::{self, Read, Write};

/// A munger which XORs a key with some data.
///
/// # Lifetime parameter `'a`
/// The struct holds a *reference* to the key (`&'a [u8]`), not an owned copy.
/// Xorcism can only live as long as the key it borrows from.
/// No heap allocation needed for the key. Nice.
#[derive(Clone)]
pub struct Xorcism<'a> {
    key: &'a [u8],
    pos: usize,
}

impl<'a> Xorcism<'a> {
    /// Create a new Xorcism munger from a key.
    ///
    /// # Generic bounds explained
    ///
    /// `Key: AsRef<[u8]>` - Accept any type that can cheaply convert to `&[u8]`.
    /// This includes String, &str, Vec<u8>, &[u8], [u8; N], etc.
    /// Basically: "if you can give me bytes, I'll take them."
    ///
    /// `Key: ?Sized` - Opt out of the implicit `Sized` bound. By default, generics
    /// require types with known compile-time size. Adding ?Sized allows unsized
    /// types like `str` and `[u8]`. This works because we only handle `&Key`
    /// (a fat pointer with known size), never `Key` directly.
    /// Fat pointer = pointer + length. The compiler knows how big *that* is.
    ///
    /// # Lifetime connection
    /// `key: &'a Key` ties the input's lifetime to the struct's lifetime parameter.
    /// When we call `key.as_ref()`, the resulting `&[u8]` inherits this lifetime.
    pub fn new<Key: AsRef<[u8]> + ?Sized>(key: &'a Key) -> Xorcism<'a> {
        Xorcism {
            key: key.as_ref(),
            pos: 0,
        }
    }

    /// XOR each byte of the input buffer with a byte from the key.
    ///
    /// # Statefulness
    /// The `pos` field tracks our position in the key. Each call continues
    /// from where the last one left off. XORing the same data twice with the
    /// same Xorcism gives different results (unless the data length is a
    /// multiple of the key length).
    ///
    /// # Cycling the key
    /// `self.pos % self.key.len()` wraps around when we reach the end of the key.
    /// For a 5-byte key and 12-byte data: key bytes 0,1,2,3,4,0,1,2,3,4,0,1
    pub fn munge_in_place(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte ^= self.key[self.pos % self.key.len()];
            self.pos += 1;
        }
    }

    /// XOR each byte of the data with a byte from the key, returning an iterator.
    ///
    /// # Generic bounds explained
    ///
    /// `Data: IntoIterator` - Accept anything that can become an iterator
    /// (slices, Vec, arrays, other iterators, etc.). Very flexible.
    ///
    /// `Data::Item: Borrow<u8>` - The iterator's items must be borrowable as u8.
    /// This abstracts over `u8` (owned) and `&u8` (borrowed). When you iterate
    /// over `&[u8]`, you get `&u8` items. When you iterate over `Vec<u8>`, you
    /// get `u8` items. Borrow handles both via `.borrow()`. Polymorphism baby.
    ///
    /// # Lifetime bounds explained
    ///
    /// This is where it gets spicy:
    /// - `&'b mut self` - We borrow self mutably for lifetime 'b
    /// - `Data: 'b` - The data must live at least as long as 'b
    /// - `impl Iterator + 'b` - The returned iterator lives for 'b
    ///
    /// Translation: the iterator borrows self, so you can't use self again
    /// until the iterator is dropped. The `+ 'b` in the return type makes this
    /// explicit to the compiler. "Hey compiler, this thing holds onto self."
    ///
    /// # Closure capture
    /// `move |byte|` moves self (which is `&'b mut self`, a mutable reference)
    /// into the closure. The closure can then mutate self.pos as bytes are consumed.
    /// The reference moves, not the whole struct. References are just pointers.
    pub fn munge<'b, Data>(&'b mut self, data: Data) -> impl Iterator<Item = u8> + 'b
    where
        Data: IntoIterator + 'b,
        Data::Item: Borrow<u8>,
    {
        data.into_iter().map(move |byte| {
            let key_byte = self.key[self.pos % self.key.len()];
            self.pos += 1;
            *byte.borrow() ^ key_byte
        })
    }

    /// Wrap a reader to XOR bytes as they are read.
    ///
    /// # Ownership
    /// Takes `self` by value (not &self or &mut self), consuming the Xorcism.
    /// The Xorcism becomes part of the XorReader wrapper. It's gone. Moved. Bye.
    #[cfg(feature = "io")]
    pub fn reader<R: Read>(self, reader: R) -> XorReader<'a, R> {
        XorReader { xor: self, reader }
    }

    /// Wrap a writer to XOR bytes as they are written.
    #[cfg(feature = "io")]
    pub fn writer<W: Write>(self, writer: W) -> XorWriter<'a, W> {
        XorWriter { xor: self, writer }
    }
}

// =============================================================================
// I/O adapters for streaming XOR operations
// =============================================================================
//
// These types are only compiled when the "io" feature is enabled.
// `#[cfg(feature = "io")]` is conditional compilation.
// Keeps binary size smaller when I/O isn't needed. Cargo features are neat.

/// A reader wrapper that XORs bytes as they're read from the underlying source.
///
/// # Type parameters
/// - 'a: Lifetime of the key reference inside the Xorcism
/// - R: The underlying reader type (anything implementing std::io::Read)
#[cfg(feature = "io")]
pub struct XorReader<'a, R> {
    xor: Xorcism<'a>,
    reader: R,
}

/// Implement the Read trait for our wrapper.
///
/// # Trait bounds on impl
/// `R: Read` is required here because we call `self.reader.read()`.
/// Without this bound, the compiler wouldn't know that R has a read method.
/// "Trust me bro" doesn't work with rustc.
#[cfg(feature = "io")]
impl<'a, R: Read> Read for XorReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Read from underlying source into the buffer
        let n = self.reader.read(buf)?;
        // XOR only the bytes that were actually read (buf[..n])
        // The rest of the buffer is untouched/undefined
        self.xor.munge_in_place(&mut buf[..n]);
        Ok(n)
    }
}

/// A writer wrapper that XORs bytes before writing them to the underlying sink.
#[cfg(feature = "io")]
pub struct XorWriter<'a, W> {
    xor: Xorcism<'a>,
    writer: W,
}

#[cfg(feature = "io")]
impl<'a, W: Write> Write for XorWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // # Stack buffer to avoid heap allocation
        //
        // Problem: `buf` is `&[u8]` (immutable), but munge_in_place needs `&mut [u8]`.
        // We can't mutate the input directly. Rust says no.
        //
        // Solution: Copy to a temporary buffer, munge it, then write.
        //
        // Why stack instead of heap (Vec)?
        // - Stack allocation is essentially free (just move the stack pointer)
        // - No malloc/free overhead
        // - The exercise emphasizes avoiding heap allocation
        // - We're cool like that
        //
        // Trade-off: Fixed size means we might write fewer bytes than requested.
        // This is fine! Write::write is allowed to write fewer bytes than buf.len().
        // The caller is responsible for calling write again with remaining data
        // (or using write_all which loops for you). This is the Write contract.
        let mut stack_buffer = [0u8; 1024];
        let len = buf.len().min(stack_buffer.len());
        stack_buffer[..len].copy_from_slice(&buf[..len]);
        self.xor.munge_in_place(&mut stack_buffer[..len]);
        self.writer.write(&stack_buffer[..len])
    }

    fn flush(&mut self) -> io::Result<()> {
        // Delegate to underlying writer. We don't buffer anything ourselves.
        self.writer.flush()
    }
}
