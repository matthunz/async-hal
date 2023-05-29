use crate::io;
use futures::{Sink, Stream};

#[cfg(feature = "nb")]
mod reader;
#[cfg(feature = "nb")]
pub use reader::Reader;

#[cfg(feature = "nb")]
mod writer;
#[cfg(feature = "nb")]
pub use writer::Writer;

pub trait Serial: SerialRead + SerialWrite {}

impl<T> Serial for T where T: ?Sized + SerialRead + SerialWrite {}

pub trait SerialRead: Stream<Item = Result<u8, Self::Error>> + serial_read::Sealed {
    type Error;

    fn into_reader(self) -> io::Reader<Self>
    where
        Self: Sized + Unpin,
    {
        io::reader(self)
    }
}

impl<T, E> SerialRead for T
where
    T: ?Sized + Stream<Item = Result<u8, E>>,
{
    type Error = E;
}

pub trait SerialWrite: Sink<u8> + serial_write::Sealed {
    fn into_writer(self) -> io::Writer<Self>
    where
        Self: Sized + Unpin,
    {
        io::writer(self)
    }
}

impl<T> SerialWrite for T where T: ?Sized + Sink<u8> {}

mod serial_read {
    use super::Stream;

    pub trait Sealed {}

    impl<T, E> Sealed for T where T: ?Sized + Stream<Item = Result<u8, E>> {}
}

mod serial_write {
    use super::Sink;

    pub trait Sealed {}

    impl<T> Sealed for T where T: ?Sized + Sink<u8> {}
}
