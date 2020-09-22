pub(crate) struct Reader<'r> {
    inner: &'r [u8],
    position: usize,
    len: usize,
}
impl<'r> Reader<'r> {
    pub(crate) fn new(text: &'r str) -> Self {
        Reader {
            inner: text.as_bytes(),
            position: 0,
            len: text.len(),
        }
    }

    #[inline]
    pub(crate) fn current(&mut self) -> char {
        assert!(self.position < self.len);
        self.inner[self.position] as char
    }

    #[inline]
    pub(crate) fn next(&mut self) -> Option<char> {
        if self.position < self.len - 1 {
            self.position += 1;
            return Some(self.current());
        }

        None
    }

    #[inline]
    pub(crate) fn peek(&self) -> Option<char> {
        if self.position < self.len - 1 {
            return Some(self.inner[self.position + 1] as char);
        }

        None
    }

    #[inline]
    pub(crate) fn skip(&mut self, n: usize) {
        if self.position + n >= self.len {
            self.position = self.len - 1;
        } else {
            self.position += n;
        }
    }

    #[inline]
    pub(crate) fn skip_whitespace(&mut self) {
        while self.current().is_ascii_whitespace() {
            if let None = self.next() {
                break;
            }
        }
    }

    #[inline]
    pub(crate) fn is_last(&self) -> bool {
        self.position == (self.len - 1)
    }
}
