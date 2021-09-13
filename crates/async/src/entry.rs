// plctag-rs
//
// a rust wrapper of libplctag, with rust style APIs and useful extensions.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::*;
use tokio::time;
use tokio::time::Duration;

#[derive(Debug)]
pub struct TagEntry {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    tag: RawTag,
    lock: tokio::sync::Mutex<()>,
}

impl Clone for TagEntry {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl TagEntry {
    pub async fn create(options: impl Into<String>) -> Result<Self> {
        let path = options.into();
        let tag = {
            let path = path.clone();
            task::spawn_blocking(move || RawTag::new(path, 0)).await??
        };
        loop {
            let status = tag.status();
            if status.is_pending() {
                //task::yield_now().await;
                time::sleep(Duration::from_millis(1)).await;
                continue;
            }
            if status.is_err() {
                status.into_result()?;
            }
            //is ok
            break;
        }
        Ok(Self {
            inner: Arc::new(Inner {
                tag,
                lock: tokio::sync::Mutex::new(()),
            }),
        })
    }

    pub async fn get(&self) -> Result<TagRef<'_>> {
        let lock = self.inner.lock.lock().await;
        let tag = &self.inner.tag;
        Ok(TagRef { tag, lock })
    }
}

impl From<RawTag> for TagEntry {
    #[inline(always)]
    fn from(tag: RawTag) -> Self {
        Self {
            inner: Arc::new(Inner {
                tag,
                lock: tokio::sync::Mutex::new(()),
            }),
        }
    }
}