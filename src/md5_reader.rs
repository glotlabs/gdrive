use bytes::buf::Reader;
use bytes::Buf;
use std::io;
use std::io::Read;

pub struct Md5Reader<T> {
    pub context: md5::Context,
    pub reader: Reader<T>,
}

impl<T> Md5Reader<T> {
    pub fn md5(self) -> String {
        format!("{:x}", self.context.compute())
    }
}

impl<T: Buf> Read for Md5Reader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let byte_count = self.reader.read(buf)?;
        self.context.consume(&buf[..byte_count]);
        Ok(byte_count)
    }
}
