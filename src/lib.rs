#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

/// A circular buffer. It is a fixed size,
/// and new writes overwrite older data, such that for a buffer
/// of size N, for any amount of writes, only the last N bytes
/// are retained.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Buffer<B> {
  data: B,
  write_cursor: usize,
  written: usize,
}

impl<B> From<B> for Buffer<B> {
  fn from(data: B) -> Self {
    Self {
      data,
      write_cursor: 0,
      written: 0,
    }
  }
}

impl<B> Buffer<B> {
  /// Creates a new buffer with the given data.
  #[inline]
  pub const fn new(data: B) -> Self {
    Self {
      data,
      write_cursor: 0,
      written: 0,
    }
  }

  /// Writes up to len(buf) bytes to the internal ring,
  /// overriding older data if necessary.
  pub fn write(&mut self, mut buf: &[u8]) -> usize
  where
    B: AsMut<[u8]>,
  {
    // Account for total bytes written
    let n = buf.len();
    let data = self.data.as_mut();
    let size = data.len();
    self.written += n;

    // If the buffer is larger than ours, then we only care
    // about the last size bytes anyways
    if n > size {
      buf = &buf[n - size..];
    }

    // Copy in place
    let remain = size - self.write_cursor;
    let data = self.data.as_mut();
    copy(&mut data[self.write_cursor..], buf);
    if n > remain {
      copy(data, &buf[remain..]);
    }

    // Update location of the cursor
    self.write_cursor = (self.write_cursor + buf.len()) % size;
    n
  }

  /// Returns how many bytes can be read from the buffer.
  ///
  /// This is useful when you want to read from the buffer.
  #[inline]
  pub fn read_hint(&self) -> usize
  where
    B: AsRef<[u8]>,
  {
    let data = self.data.as_ref();
    let size = data.len();
    match () {
      () if self.written >= size && self.write_cursor == 0 => self.data.as_ref().len(),
      () if self.written > size => size,
      _ => self.data.as_ref()[..self.write_cursor].len(),
    }
  }

  /// Reads the whole buffer into the `dst`, returns number of bytes readed.
  ///
  /// To avoid panics, you should check the [`read_hint`](Buffer::read_hint) method
  /// to see how many bytes can be read.
  ///
  /// ## Panics
  ///
  /// Panics if the data contained in the buffer is larger than the given `dst`.
  ///
  pub fn read_into(&self, dst: &mut [u8]) -> usize
  where
    B: AsRef<[u8]>,
  {
    let data = self.data.as_ref();
    let size = data.len();

    match () {
      () if self.written >= size && self.write_cursor == 0 => {
        dst[..size].copy_from_slice(data);
        size
      }
      () if self.written > size => {
        copy(dst, &data[self.write_cursor..]);
        copy(
          &mut dst[size - self.write_cursor..],
          &data[..self.write_cursor],
        );
        size
      }
      _ => {
        dst[..self.write_cursor].copy_from_slice(&data[..self.write_cursor]);
        self.write_cursor
      }
    }
  }

  /// Provides a slice of the bytes written. This
  /// slice should not be written to.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
  #[inline]
  pub fn read_to_bytes(&self) -> std::borrow::Cow<'_, [u8]>
  where
    B: AsRef<[u8]>,
  {
    let data = self.data.as_ref();
    let size = data.len();
    std::borrow::Cow::Borrowed(match () {
      () if self.written >= size && self.write_cursor == 0 => data,
      () if self.written > size => {
        let mut out = std::vec![0; size];
        copy(&mut out, &data[self.write_cursor..]);
        copy(
          &mut out[size - self.write_cursor..],
          &data[..self.write_cursor],
        );
        return out.into();
      }
      _ => &data[..self.write_cursor],
    })
  }

  /// Returns the size of the buffer
  #[inline]
  pub fn size(&self) -> usize
  where
    B: AsRef<[u8]>,
  {
    self.data.as_ref().len()
  }

  /// Returns the total number of bytes written to the buffer
  #[inline]
  pub const fn written(&self) -> usize {
    self.written
  }

  /// Resets the buffer so it has no content.
  #[inline]
  pub const fn reset(&mut self) {
    self.write_cursor = 0;
    self.written = 0;
  }

  /// Consumes the buffer and returns the underlying data.
  #[inline]
  pub fn into_inner(self) -> B {
    self.data
  }
}

#[cfg(feature = "std")]
const _: () = {
  use std::io::Write;

  impl<B> Write for Buffer<B>
  where
    B: AsMut<[u8]>,
  {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
      Ok(self.write(buf))
    }

    fn flush(&mut self) -> std::io::Result<()> {
      Ok(())
    }
  }
};

#[cfg(all(feature = "tokio", feature = "std"))]
const _: () = {
  use core::{
    pin::Pin,
    task::{Context, Poll},
  };
  use tokio::io::AsyncWrite;

  impl<B> AsyncWrite for Buffer<B>
  where
    B: AsMut<[u8]> + Unpin,
  {
    fn poll_write(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
      buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
      Poll::Ready(Ok(self.get_mut().write(buf)))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
      Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
      Poll::Ready(Ok(()))
    }
  }
};

#[cfg(all(feature = "std", feature = "futures-io"))]
const _: () = {
  use core::{
    pin::Pin,
    task::{Context, Poll},
  };
  use futures_io::AsyncWrite;

  impl<B> AsyncWrite for Buffer<B>
  where
    B: AsMut<[u8]> + Unpin,
  {
    fn poll_write(
      self: Pin<&mut Self>,
      _: &mut Context<'_>,
      buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
      Poll::Ready(Ok(self.get_mut().write(buf)))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
      Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
      Poll::Ready(Ok(()))
    }
  }
};

/// Copies elements from a source slice into a destination slice. (As a special case, it also will copy bytes from a string to a slice of bytes.) The source and destination may overlap.
/// Copy returns the number of elements copied, which will be the minimum of `src.len()` and `dst.len()`.
#[inline]
fn copy(dst: &mut [u8], src: &[u8]) -> usize {
  let min_len = core::cmp::min(src.len(), dst.len());
  dst[..min_len].copy_from_slice(&src[..min_len]);
  min_len
}
