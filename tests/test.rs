use circularbuf::Buffer;

#[test]
fn api() {
  let buf = Buffer::<[u8; 1]>::from([0]);
  assert_eq!(buf.size(), 1);
  assert_eq!(buf.written(), 0);
  assert_eq!([0u8], buf.into_inner());
}

#[test]
fn short_write() {
  let mut buf = Buffer::new([0u8; 1024]);

  let inp = b"hello world";

  let n = buf.write(inp);
  assert_eq!(n, inp.len());

  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), inp);

  assert_eq!(buf.read_hint(), n);

  let mut read_buf = [0u8; 1024];
  let n = buf.read_into(&mut read_buf);
  assert_eq!(n, inp.len());
}

#[test]
fn full_write() {
  const BUF: &[u8] = b"hello world";

  let mut buf = Buffer::new([0u8; BUF.len()]);

  let n = buf.write(BUF);
  assert_eq!(n, BUF.len());

  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), BUF);
  assert_eq!(buf.read_hint(), n);

  let mut read_buf = [0u8; BUF.len()];
  let n = buf.read_into(&mut read_buf);
  assert_eq!(n, BUF.len());
}

#[test]
fn long_write() {
  const BUF: &[u8] = b"hello world";

  let mut buf = Buffer::new([0u8; 6]);

  let n = buf.write(BUF);
  assert_eq!(n, BUF.len());

  let expect = b" world";
  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), expect);
  assert_eq!(buf.read_hint(), expect.len());

  let mut read_buf = [0u8; 6];
  let n = buf.read_into(&mut read_buf);
  assert_eq!(n, expect.len());
}

#[test]
fn huge_write() {
  const BUF: &[u8] = b"hello world";

  let mut buf = Buffer::new([0u8; 3]);

  let n = buf.write(BUF);
  assert_eq!(n, BUF.len());

  let expect = b"rld";
  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), expect);
  assert_eq!(buf.read_hint(), expect.len());

  let mut read_buf = [0u8; 6];
  let n = buf.read_into(&mut read_buf);
  assert_eq!(n, expect.len());
}

#[test]
fn many_small() {
  const BUF: &[u8] = b"hello world";

  let mut buf = Buffer::new([0u8; 3]);

  for c in BUF.iter() {
    let n = buf.write(&[*c]);
    assert_eq!(n, 1);
  }

  let expect = b"rld";
  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), expect);

  let mut read_buf = [0u8; 6];
  let n = buf.read_into(&mut read_buf);
  assert_eq!(n, expect.len());
}

#[test]
fn multi_part() {
  const INPUTS: &[&[u8]] = &[b"hello world\n", b"this is a test\n", b"my cool input\n"];

  let mut total = 0;

  let mut buf = Buffer::new([0u8; 16]);

  for inp in INPUTS.iter() {
    let n = buf.write(inp);
    assert_eq!(n, inp.len());
    total += n;
  }

  assert_eq!(total, buf.written());

  let expect = b"t\nmy cool input\n";

  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), expect);

  let mut read_buf = [0u8; 16];
  let n = buf.read_into(&mut read_buf);
  assert_eq!(n, expect.len());
}

#[test]
fn reset() {
  const INPUTS: &[&[u8]] = &[b"hello world\n", b"this is a test\n", b"my cool input\n"];

  let mut buf = Buffer::new([0u8; 4]);

  for inp in INPUTS.iter() {
    let n = buf.write(inp);
    assert_eq!(n, inp.len());
  }

  // Reset it
  buf.reset();

  // Write more data
  let n = buf.write(b"hello");

  assert_eq!(n, 5);

  // Test the output
  let expect = b"ello";

  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), expect);

  let mut read_buf = [0u8; 4];
  let n = buf.read_into(&mut read_buf);
  assert_eq!(n, expect.len());
}

#[test]
#[cfg(feature = "std")]
fn io_write() {
  use std::io::Write;

  let mut buf = Buffer::new([0u8; 1024]);

  let inp = b"hello world";
  Write::write_all(&mut buf, inp).unwrap();
  buf.flush().unwrap();

  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), inp);
}

#[tokio::test]
#[cfg(feature = "tokio")]
async fn tokio_io_write() {
  use tokio::io::AsyncWriteExt;

  let mut buf = Buffer::new([0u8; 1024]);

  let inp = b"hello world";
  buf.write_all(inp).await.unwrap();
  buf.flush().await.unwrap();
  buf.shutdown().await.unwrap();

  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), inp);
}

#[tokio::test]
#[cfg(feature = "future")]
async fn futures_io_write() {
  use futures_util::AsyncWriteExt;

  let mut buf = Buffer::new([0u8; 1024]);

  let inp = b"hello world";
  buf.write_all(inp).await.unwrap();
  buf.flush().await.unwrap();
  buf.close().await.unwrap();

  let out = buf.read_to_bytes();
  assert_eq!(out.as_ref(), inp);
}
