use std::io::{BufWriter, Write};

pub struct Writer<'a, W: Write>(BufWriter<&'a mut W>);

impl<'a, W: Write> Writer<'a, W> {
    pub fn new(term: &'a mut W) -> Self {
        Writer(BufWriter::new(term))
    }

    /// Clear the entire screen
    pub fn clear_screen(&mut self) {
        // set scroll region
        self.write_all(b"\x1b[;r").unwrap();
        // clear the screen
        self.write_all(b"\x1b[2J").unwrap()
    }

    /// Scroll up by 'n' lines
    pub fn scroll(&mut self, n: usize) {
        self.write_all(&["\x1b[", &n.to_string(), "S"].concat().as_bytes())
            .unwrap()
    }

    pub fn clear_line(&mut self) {
        self.write_all(b"\x1b[2k").unwrap()
    }

    /// Goto 'row' and 'col'
    pub fn goto(&mut self, row: usize, col: usize) {
        let (row, col) = (row + 1, col + 1);
        self.write_all(
            &["\x1b[", &row.to_string(), ";", &col.to_string(), "H"]
                .concat()
                .as_bytes(),
        )
        .unwrap();
    }

    /// Move the cursor to row + 1 and to col * 0
    pub fn carriage_return(&mut self) {
        self.write_all(b"\x1b[1E").unwrap();
    }
}

impl<'a, W: Write> Write for Writer<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write_all(buf).map(|_| buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> Drop for Writer<'a, W> {
    fn drop(&mut self) {
        let _ = self.0.flush();
    }
}
