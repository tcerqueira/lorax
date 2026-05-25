use std::io::{self, BufRead, Seek, SeekFrom, Write};

use thiserror::Error;

pub trait Decode: Sized {
    fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError>;
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("unknown op code: {0}")]
    UnknownOpCode(u8),
    #[error("needed {needed} bytes, found {available}")]
    InsufficientBytes { needed: usize, available: usize },
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub trait OpDecoder: BufRead + Seek {
    fn decode_op<T>(&mut self) -> Result<Option<T>, DecodeError>
    where
        T: Decode,
    {
        let buf = self.fill_buf()?;
        if buf.is_empty() {
            return Ok(None);
        }

        let (opcode, consumed) = T::decode(buf)?;
        self.consume(consumed);
        Ok(Some(opcode))
    }

    #[expect(dead_code)]
    fn current_position(&mut self) -> io::Result<u64> {
        self.stream_position()
    }
    #[expect(dead_code)]
    fn jump_to(&mut self, addr: u64) -> io::Result<()> {
        self.seek(SeekFrom::Start(addr))?;
        Ok(())
    }
    #[expect(dead_code)]
    fn relative_jump(&mut self, offset: i64) -> io::Result<()> {
        self.seek(SeekFrom::Current(offset))?;
        Ok(())
    }
}

impl<R> OpDecoder for R where R: BufRead + Seek {}

pub trait Encode {
    fn encode<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<usize>;
}

pub trait OpEncoder: Write {
    fn encode_op<T>(&mut self, opcode: &T) -> io::Result<usize>
    where
        T: Encode,
    {
        opcode.encode(self)
    }
}

impl<W> OpEncoder for W where W: Write {}
