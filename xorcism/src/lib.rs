use std::borrow::Borrow;
#[cfg(feature = "io")]
use std::io::{self, Read, Write};

/// A munger which XORs a key with some data
#[derive(Clone)]
pub struct Xorcism<'a> {
    key: &'a [u8],
    pos: usize,
}

impl<'a> Xorcism<'a> {
    /// Create a new Xorcism munger from a key
    ///
    /// Should accept anything which has a cheap conversion to a byte slice.
    /// Optionally sized Key - we're not handling Key directly, but &Key, which is
    /// a fat pointer (pointer + length and other metadata).
    pub fn new<Key: AsRef<[u8]> + ?Sized>(key: &'a Key) -> Xorcism<'a> {
        Xorcism {
            key: key.as_ref(),
            pos: 0,
        }
    }

    /// XOR each byte of the input buffer with a byte from the key.
    ///
    /// Note that this is stateful: repeated calls are likely to produce different results,
    /// even with identical inputs.
    pub fn munge_in_place(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte ^= self.key[self.pos % self.key.len()];
            self.pos += 1;
        }
    }

    /// XOR each byte of the data with a byte from the key.
    ///
    /// Note that this is stateful: repeated calls are likely to produce different results,
    /// even with identical inputs.
    ///
    /// Should accept anything which has a cheap conversion to a byte iterator.
    /// Shouldn't matter whether the byte iterator's values are owned or borrowed.
    ///
    /// Lifetime bounds:
    /// - borrow self for 'b
    /// - data must live at least as long as 'b
    /// - the returned iterator lives for 'b
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

// I/O adapters for streaming XOR operations

#[cfg(feature = "io")]
pub struct XorReader<'a, R> {
    xor: Xorcism<'a>,
    reader: R,
}

#[cfg(feature = "io")]
impl<'a, R: Read> Read for XorReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.xor.munge_in_place(&mut buf[..n]);
        Ok(n)
    }
}

#[cfg(feature = "io")]
pub struct XorWriter<'a, W> {
    xor: Xorcism<'a>,
    writer: W,
}

#[cfg(feature = "io")]
impl<'a, W: Write> Write for XorWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut tmp = [0u8; 1024];
        let len = buf.len().min(tmp.len());
        tmp[..len].copy_from_slice(&buf[..len]);
        self.xor.munge_in_place(&mut tmp[..len]);
        self.writer.write(&tmp[..len])
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
