# Asynchronous streams for Rust

Asynchronous stream of elements.

Provides two macros, `stream!` and `try_stream!`, allowing the caller to
define asynchronous streams of elements. These are implemented using `async`
& `await` notation. The `stream!` macro works without unstable features.

The `stream!` macro returns an anonymous type implementing the [`Stream`]
trait. The `Item` associated type is the type of the values yielded from the
stream. The `try_stream!` also returns an anonymous type implementing the
[`Stream`] trait, but the `Item` associated type is `Result<T, Error>`. The
`try_stream!` macro supports using `?` notiation as part of the
implementation.

## Usage

A basic stream yielding numbers. Values are yielded using the `yield`
keyword. The stream block must return `()`.

```rust
use tokio::prelude::*;

use async_stream::stream;
use futures_util::pin_mut;

#[tokio::main]
async fn main() {
    let s = stream! {
        for i in 0..3 {
            yield i;
        }
    };

    pin_mut!(s); // needed for iteration

    while let Some(value) = s.next().await {
        println!("got {}", value);
    }
}
```

Streams may be returned by using `impl Stream<Item = T>`:

```rust
use tokio::prelude::*;

use async_stream::stream;
use futures_util::pin_mut;

fn zero_to_three() -> impl Stream<Item = u32> {
    stream! {
        for i in 0..3 {
            yield i;
        }
    }
}

#[tokio::main]
async fn main() {
    let s = zero_to_three();
    pin_mut!(s); // needed for iteration

    while let Some(value) = s.next().await {
        println!("got {}", value);
    }
}
```

Streams may be implemented in terms of other streams:

```rust
use tokio::prelude::*;

use async_stream::stream;
use futures_util::pin_mut;

fn zero_to_three() -> impl Stream<Item = u32> {
    stream! {
        for i in 0..3 {
            yield i;
        }
    }
}

fn double<S: Stream<Item = u32>>(input: S)
    -> impl Stream<Item = u32>
{
    stream! {
        pin_mut!(input);
        while let Some(value) = input.next().await {
            yield value * 2;
        }
    }
}

#[tokio::main]
async fn main() {
    let s = double(zero_to_three());
    pin_mut!(s); // needed for iteration

    while let Some(value) = s.next().await {
        println!("got {}", value);
    }
}
```

Rust try notation (`?`) can be used with the `try_stream!` macro. The `Item`
of the returned stream is `Result` with `Ok` being the value yielded and
`Err` the error type returned by `?`.

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

use async_stream::try_stream;
use std::io;
use std::net::SocketAddr;

fn bind_and_accept(addr: SocketAddr)
    -> impl Stream<Item = io::Result<TcpStream>>
{
    try_stream! {
        let mut listener = TcpListener::bind(&addr)?;

        loop {
            let (stream, addr) = listener.accept().await?;
            println!("received on {:?}", addr);
            yield stream;
        }
    }
}
```

## Implementation

The `stream!` and `try_stream!` macros are implemented using proc macros.
Given that proc macros in expression position are not supported on stable
rust, a hack similar to the one provided by the [`proc-macro-hack`] crate is
used. The macro searches the syntax tree for instances of `sender.send($expr)` and
transforms them into `sender.send($expr).await`.

The stream uses a lightweight sender to send values from the stream
implementation to the caller. When entering the stream, an `Option<T>` is
stored on the stack. A pointer to the cell is stored in a thread local and
`poll` is called on the async block. When `poll` returns.
`sender.send(value)` stores the value that cell and yields back to the
caller.

## Limitations

`async-stream` suffers from the same limitations as the [`proc-macro-hack`]
crate. Primarily, nesting support must be implemented using a `TT-muncher`.
If large `stream!` blocks are used, the caller will be required to add
`#![recursion_limit = "..."]` to their crate.

A `stream!` macro may only contain up to 64 macro invocations.

[`Stream`]: https://docs.rs/futures-core-preview/*/futures_core/stream/trait.Stream.html
[`proc-macro-hack`]: https://github.com/dtolnay/proc-macro-hack/

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `async-stream` by you, shall be licensed as MIT, without any
additional terms or conditions.
