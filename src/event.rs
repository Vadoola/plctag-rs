use crate::raw::{Accessor, RawTag, TagId};
use crate::{Result, Status};
use futures::prelude::*;
use parking_lot;
use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::prelude::*;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time;

lazy_static! {
    static ref TAGS: parking_lot::Mutex<HashMap<i32, Holder>> =
        parking_lot::Mutex::new(HashMap::new());
}

struct Holder {
    ptr: *mut EventTag,
}

impl Deref for Holder {
    type Target = EventTag;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for Holder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

unsafe impl Send for Holder {}

fn global_register(tag: &mut EventTag) {
    let mut data = TAGS.lock();
    let map = &mut *data;
    map.insert(
        tag.raw.id(),
        Holder {
            ptr: tag as *mut EventTag,
        },
    );
    unsafe {
        tag.raw.register_callback(Some(on_tag_event));
    }
}

fn global_unregister(tag: &mut EventTag) {
    let mut data = TAGS.lock();
    let map = &mut *data;
    map.remove(&tag.raw.id());
    tag.raw.unregister_callback();
}

extern "C" fn on_tag_event(tag_id: i32, event: i32, status: i32) {
    let data = TAGS.lock();
    let map = &*data;
    if let Some(tag) = map.get(&tag_id) {
        EventTag::on_event(&tag, event, status);
    }
}

pub struct EventTag {
    raw: Arc<RawTag>,
    created: Option<Status>,
}

impl EventTag {
    pub fn new(raw: RawTag) -> Self {
        let mut tag = Self {
            raw: Arc::new(raw),
            created: None,
        };
        global_register(&mut tag);
        tag
    }

    pub async fn status(&self) -> Status {
        let raw = self.raw.clone();
        let res = asyncify(move || Ok(raw.status())).await;
        match res {
            Ok(status) => status,
            Err(status) => status,
        }
    }

    async fn wait_async(&self, duration: Duration) -> Status {
        let now = Instant::now();
        loop {
            if now.elapsed() > duration {
                return Status::err_timeout();
            }
            let status = self.status().await;
            if !status.is_pending() {
                return status;
            }
            time::delay_for(Duration::from_millis(1)).await;
        }
    }

    async fn read_async(&self) -> Status {
        let raw = self.raw.clone();
        let res = asyncify(move || Ok(raw.read(0))).await;
        match res {
            Ok(status) => status,
            Err(status) => status,
        }
    }

    async fn write_async(&self) -> Status {
        let raw = self.raw.clone();
        let res = asyncify(move || Ok(raw.write(0))).await;
        match res {
            Ok(status) => status,
            Err(status) => status,
        }
    }

    async fn abort_async(&self) -> Status {
        let raw = self.raw.clone();
        let res = asyncify(move || Ok(raw.abort())).await;
        match res {
            Ok(_) => Status::Ok,
            Err(status) => status,
        }
    }

    pub async fn created(&mut self) -> Status {
        //until hack the `libplc`, no way to be notified when creation completed
        if let Some(status) = self.created {
            status
        } else {
            loop {
                let status = self.status().await;
                if !status.is_pending() {
                    self.created = Some(status);
                    return status;
                }
                time::delay_for(Duration::from_millis(1)).await;
            }
        }
    }

    pub async fn read(&mut self, duration: Duration) -> Status {
        let status = self.created().await;
        if !status.is_ok() {
            return status;
        }

        let status = self.read_async().await;
        if !status.is_pending() {
            return status;
        }
        let status = self.wait_async(duration).await;
        if status.is_timeout() {
            //timeout needs to abort pending operation
            self.abort_async().await;
        }
        status
    }
    pub async fn write(&mut self, duration: Duration) -> Status {
        let status = self.created().await;
        if !status.is_ok() {
            return status;
        }

        let status = self.write_async().await;
        if !status.is_pending() {
            return status;
        }
        let status = self.wait_async(duration).await;
        if status.is_timeout() {
            //timeout needs to abort pending operation
            self.abort_async().await;
        }
        status
    }

    fn on_event(&self, event: i32, status: i32) {
        println!("tag {} event {} status {}", self.raw.id(), event, status);
    }
}

impl Drop for EventTag {
    fn drop(&mut self) {
        global_unregister(self);
    }
}

impl Deref for EventTag {
    type Target = RawTag;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

// impl DerefMut for EventTag {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.raw
//     }
// }

pub(crate) async fn asyncify<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    match task::spawn_blocking(f).await {
        Ok(res) => res,
        Err(_) => Err(Status::err_task()),
    }
}
