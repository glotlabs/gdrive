use std::io;
use std::io::Write;

pub struct Md5Writer<T> {
    writer: T,
    context: md5::Context,
}

impl<T: Write> Md5Writer<T> {
    pub fn new(writer: T) -> Self {
        Self {
            writer,
            context: md5::Context::new(),
        }
    }

    pub fn md5(self) -> String {
        format!("{:x}", self.context.compute())
    }
}

impl<T: Write> Write for Md5Writer<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let byte_count = self.writer.write(buf)?;
        self.context.consume(&buf[..byte_count]);
        Ok(byte_count)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
