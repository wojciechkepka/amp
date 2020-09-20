pub(crate) struct Reader<'r> {
    inner: &'r [u8],
    position: usize,
    len: usize,
    hooks: Vec<usize>,
}
impl<'r> Reader<'r> {
    pub(crate) fn new(text: &'r str) -> Self {
        Reader { inner: text.as_bytes(), position: 0, len: text.len(), hooks: Vec::new() }
    }

    pub(crate) fn save_hook(&mut self) {
        self.hooks.push(self.position);
    }

    pub(crate) fn rewind_last_hook(&mut self) {
        if let Some(hook) = self.hooks.last() {
            self.position = *hook;
        }
    }

    #[inline]
    pub(crate) fn current(&mut self) -> char {
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
    pub(crate) fn rewind(&mut self, n: isize) {
        if (self.position as isize - n) < 0 {
            self.position = 0;
        } else {
            self.position -= n as usize;
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
