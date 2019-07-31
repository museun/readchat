use std::io::{BufWriter, Stdout, Write};

/// Crossterm has poor writing, this is faster
pub struct BufferedWriter(BufWriter<Stdout>);

impl BufferedWriter {
    /// Make a new BufferedWriter from stdout
    pub fn stdout() -> Self {
        BufferedWriter(BufWriter::new(std::io::stdout()))
    }
    /// Clear the entire screen
    pub fn clear_screen(&mut self) {
        self.goto(0, 0);
        self.write_all(b"\x1b[0J").unwrap();
    }
    /// Hide the curosr
    pub fn hide_cursor(&mut self) {
        self.write_all(b"\x1b[25l").unwrap();
    }
    /// Show the cursor
    pub fn show_cursor(&mut self) {
        self.write_all(b"\x1b[25h").unwrap();
    }
    /// Scroll up by 'n' lines
    pub fn scroll(&mut self, n: usize) {
        self.write_all(&["\x1b[", &n.to_string(), "S"].concat().as_bytes())
            .unwrap()
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
}

impl Write for BufferedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write_all(buf).map(|_| buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
