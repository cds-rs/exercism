use std::borrow::Borrow;

/// A munger which XORs a key with some data
#[derive(Clone)]
pub struct Xorcism<'a> {
    // phantom: std::marker::PhantomData<&'a u8>,
    key: &'a [u8],
    pos: usize,
}

impl<'a> Xorcism<'a> {
    /// Create a new Xorcism munger from a key
    ///
    /// Should accept anything which has a cheap conversion to a byte slice.
    /// optionally size Key. we're not handling Key directly, but &Key, which is
    /// a fat pointer, (pointer + length and other metadata)
    pub fn new<Key: AsRef<[u8]> +?Sized>(key: &'a Key) -> Xorcism<'a> {
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
    /// shared, bound lifetimes
    /// borrow self for 'b
    /// data must live at least as long as 'b 
    /// the returned iterator lives for 'b '
    pub fn munge<'b, Data>(&'b mut self, data: Data) -> impl Iterator<Item = u8> + 'b 
    where
        Data: IntoIterator + 'b,
        Data::Item: Borrow<u8> {

        data.into_iter().map(move |byte| {
            let key_byte = self.key[self.pos % self.key.len()];
            self.pos += 1;
            *byte.borrow() ^ key_byte
        })
    }
}
