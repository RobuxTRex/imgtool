/// Walks through a byte buffer, reading it sequentially, keeping a cursor
/// to keep track of direction.
pub struct Scanner<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Scanner<'a> {
    /// Constructs a new [Scanner] using a supplied byte buffer.
    #[inline]
    pub fn new(bytes: &'a [u8]) -> Scanner<'a> {
        Self { bytes, position: 0 }
    }

    /// Reads a constant number of bytes, `N`, from the buffer.
    ///
    /// An `Option<T>` is returned because there is no guarantee the buffer can supply `N`
    /// bytes.
    ///
    /// ## Panics
    /// This function can panic when:
    /// - The position of the [Scanner] in the buffer overflows the platform's unsigned integer limit.
    pub fn read_buffer<const N: usize>(&mut self) -> Option<[u8; N]> {
        let res = self.bytes.get(self.position..self.position + N);
        if res.is_none() {
            return None;
        }

        self.position += N;

        let res = res.expect("unreachable"); // unreachable panic
        Some(res.try_into().expect("unreachable")) // unreachable panic
    }

    /// Skips `N` bytes by adding `N` to the cursor.
    #[inline]
    pub fn skip<const N: usize>(&mut self) {
        self.position += N;
    }
}
