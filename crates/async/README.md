# plctag-async

async wrapper for `libplctag`.

[![crates.io](https://img.shields.io/crates/v/plctag-async.svg)](https://crates.io/crates/plctag-async)
[![docs](https://docs.rs/plctag-async/badge.svg)](https://docs.rs/plctag-async)
[![build](https://github.com/joylei/plctag-rs/workflows/build/badge.svg?branch=master)](https://github.com/joylei/plctag-rs/actions?query=workflow%3A%22build%22)
[![license](https://img.shields.io/crates/l/plctag.svg)](https://github.com/joylei/plctag-rs/blob/master/LICENSE)

## How to use

Add `plctag-async` to your Cargo.toml

```toml
[dependencies]
plctag-async= "0.3"
```

## Examples

```rust
use plctag_async::{AsyncTag, Error, Pool, PoolEntry};
use tokio::runtime;

let rt = runtime::Runtime::new().unwrap()?;
rt.block_on(async {
   let path="protocol=ab-eip&plc=controllogix&path=1,0&gateway=192.168.1.120&name=MyTag1&elem_count=1&elem_size=16";// YOUR TAG DEFINITION
   let pool = Pool::new();
   let tag = pool.entry(path).await.unwrap();
   let tag_ref = tag.get().await.unwrap();
   let offset = 0;
   let value:u16 = tag_ref.read_value(offset).await.unwrap();
   println!("tag value: {}", value);

   let value = value + 10;
   tag_ref.write_value(offset, value).await.unwrap();
});
```

## Build

Please refer to [How to build](https://github.com/Joylei/plctag-rs/tree/master/crates/sys#build) to setup build environment.

## License

MIT
