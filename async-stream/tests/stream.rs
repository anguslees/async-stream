use async_stream::stream;

use futures_util::pin_mut;
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio_test::assert_ok;

#[tokio::test]
async fn noop_stream() {
    let s = stream! {};
    pin_mut!(s);

    while let Some(_) = s.next().await {
        unreachable!();
    }
}

#[tokio::test]
async fn empty_stream() {
    let mut ran = false;

    {
        let r = &mut ran;
        let s = stream! {
            *r = true;
            println!("hello world!");
        };
        pin_mut!(s);

        while let Some(_) = s.next().await {
            unreachable!();
        }
    }

    assert!(ran);
}

#[tokio::test]
async fn yield_single_value() {
    let s = stream! {
        yield "hello";
    };

    let values: Vec<_> = s.collect().await;

    assert_eq!(1, values.len());
    assert_eq!("hello", values[0]);
}

#[tokio::test]
async fn yield_multi_value() {
    let s = stream! {
        yield "hello";
        yield "world";
        yield "dizzy";
    };

    let values: Vec<_> = s.collect().await;

    assert_eq!(3, values.len());
    assert_eq!("hello", values[0]);
    assert_eq!("world", values[1]);
    assert_eq!("dizzy", values[2]);
}

#[tokio::test]
async fn return_stream() {
    fn build_stream() -> impl Stream<Item = u32> {
        stream! {
            yield 1;
            yield 2;
            yield 3;
        }
    }

    let s = build_stream();

    let values: Vec<_> = s.collect().await;
    assert_eq!(3, values.len());
    assert_eq!(1, values[0]);
    assert_eq!(2, values[1]);
    assert_eq!(3, values[2]);
}

#[tokio::test]
async fn consume_channel() {
    let (mut tx, mut rx) = mpsc::channel(10);

    let s = stream! {
        while let Some(v) = rx.recv().await {
            yield v;
        }
    };

    pin_mut!(s);

    for i in 0..3 {
        assert_ok!(tx.send(i).await);
        assert_eq!(Some(i), s.next().await);
    }

    drop(tx);
    assert_eq!(None, s.next().await);
}

#[tokio::test]
async fn borrow_self() {
    struct Data(String);

    impl Data {
        fn stream<'a>(&'a self) -> impl Stream<Item = &str> + 'a {
            stream! {
                yield &self.0[..];
            }
        }
    }

    let data = Data("hello".to_string());
    let s = data.stream();
    pin_mut!(s);

    assert_eq!(Some("hello"), s.next().await);
}

#[tokio::test]
async fn stream_in_stream() {
    let s = stream! {
        let s = stream! {
            for i in 0..3 {
                yield i;
            }
        };

        pin_mut!(s);
        while let Some(v) = s.next().await {
            yield v;
        }
    };

    let values: Vec<_> = s.collect().await;
    assert_eq!(3, values.len());
}

#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
