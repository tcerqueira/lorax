use std::{
    error::Error,
    io::{self, BufRead, Seek, SeekFrom, Write},
};

use thiserror::Error;

pub trait Decode: Sized {
    type Err;

    fn decode(buf: &[u8]) -> Result<(Self, usize), Self::Err>;
}

#[derive(Debug, Error)]
pub enum DecodeError<E> {
    #[error("op code error: {0}")]
    OpCodeError(E),
    // #[error("end of stream")]
    // EndOfStream,
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub trait OpDecoder: BufRead + Seek {
    fn decode_op<T>(&mut self) -> Result<Option<T>, DecodeError<T::Err>>
    where
        T: Decode,
        T::Err: Error,
    {
        let buf = self.fill_buf()?;
        if buf.is_empty() {
            return Ok(None);
        }

        let (opcode, consumed) = T::decode(buf).map_err(DecodeError::OpCodeError)?;
        self.consume(consumed);
        Ok(Some(opcode))
    }

    fn current_position(&mut self) -> io::Result<u64> {
        self.stream_position()
    }

    fn jump_to(&mut self, addr: u64) -> io::Result<()> {
        self.seek(SeekFrom::Start(addr))?;
        Ok(())
    }

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

impl<W: Write> OpEncoder for W {}
