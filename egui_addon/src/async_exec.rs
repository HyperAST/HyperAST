#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub mod web {
    use js_sys::Function;
    use std::sync::{Arc, Mutex};
    use wasm_bindgen::prelude::*;

    #[derive(Debug)]
    pub enum Error {
        // #[error("JsValue {0:?}")]
        JsValue(JsValue),

        // #[error("Invalid interval handle")]
        InvalidIntervalHandle,

        // #[error("Invalid timeout handle")]
        InvalidTimeoutHandle,
    }

    impl From<JsValue> for Error {
        fn from(value: JsValue) -> Self {
            Error::JsValue(value)
        }
    }

    pub mod native {
        use super::*;

        #[wasm_bindgen]
        extern "C" {
            /// [`mod@wasm_bindgen`] binding to the native JavaScript [`setInterval()`](https://developer.mozilla.org/en-US/docs/Web/API/setInterval) function
            #[wasm_bindgen (catch, js_name = setInterval)]
            pub fn set_interval(
                closure: &Function,
                timeout: u32,
            ) -> std::result::Result<u32, JsValue>;

            /// [`mod@wasm_bindgen`] binding to the native JavaScript [`clearInterval()`](https://developer.mozilla.org/en-US/docs/Web/API/clearInterval) function
            #[wasm_bindgen (catch, js_name = clearInterval)]
            pub fn clear_interval(interval: u32) -> std::result::Result<(), JsValue>;

            /// [`mod@wasm_bindgen`] binding to the native JavaScript [`setTimeout()`](https://developer.mozilla.org/en-US/docs/Web/API/setTimeout) function
            #[wasm_bindgen (catch, js_name = setTimeout)]
            pub fn set_timeout(
                closure: &Function,
                timeout: u32,
            ) -> std::result::Result<u32, JsValue>;

            /// [`mod@wasm_bindgen`] binding to the native JavaScript [`clearTimeout()`](https://developer.mozilla.org/en-US/docs/Web/API/clearTimeout) function
            #[wasm_bindgen (catch, js_name = clearTimeout)]
            pub fn clear_timeout(interval: u32) -> std::result::Result<(), JsValue>;

            #[wasm_bindgen(js_namespace = console)]
            pub fn log(s: &str);
        }
    }

    /// JavaScript interval handle dropping which stops and clears the associated interval
    #[derive(Clone, Debug)]
    pub struct IntervalHandle(Arc<Mutex<u32>>);

    impl Drop for IntervalHandle {
        fn drop(&mut self) {
            let handle = self.0.lock().unwrap();
            if *handle != 0 {
                native::clear_interval(*handle).expect("Unable to clear interval");
            }
        }
    }

    /// JavaScript timeout handle, droppping which cancels the associated timeout.
    #[derive(Clone)]
    pub struct TimeoutHandle0(Arc<Mutex<u32>>);

    impl Drop for TimeoutHandle0 {
        fn drop(&mut self) {
            let handle = self.0.lock().unwrap();
            if *handle != 0 {
                native::clear_timeout(*handle).expect("Unable to clear timeout");
            }
        }
    }

    /// Create JavaScript interval
    pub fn set_interval(
        closure: &Closure<dyn FnMut()>,
        timeout: u32,
    ) -> Result<IntervalHandle, Error> {
        let handle = native::set_interval(closure.as_ref().unchecked_ref(), timeout)?;
        Ok(IntervalHandle(Arc::new(Mutex::new(handle))))
    }

    /// Clear JavaScript interval using a handle returned by [`set_interval`]
    pub fn clear_interval(handle: &IntervalHandle) -> Result<(), Error> {
        let mut handle = handle.0.lock().unwrap();
        if *handle != 0 {
            native::clear_interval(*handle)?;
            *handle = 0;
            Ok(())
        } else {
            Err(Error::InvalidIntervalHandle)
        }
    }

    /// Create JavaScript timeout
    pub fn set_timeout(
        closure: &Closure<dyn FnMut()>,
        timeout: u32,
    ) -> Result<TimeoutHandle0, Error> {
        let handle = native::set_timeout(closure.as_ref().unchecked_ref(), timeout)?;
        Ok(TimeoutHandle0(Arc::new(Mutex::new(handle))))
    }

    /// Clear JavaScript timeout using a handle returns by [`set_timeout`]
    pub fn clear_timeout(handle: &TimeoutHandle0) -> Result<(), Error> {
        let mut handle = handle.0.lock().unwrap();
        if *handle != 0 {
            native::clear_timeout(*handle)?;
            *handle = 0;
            Ok(())
        } else {
            Err(Error::InvalidTimeoutHandle)
        }
    }

    // Keep logging "hello" every second until the resulting `Interval` is dropped.
    pub fn hello() -> IntervalHandle {
        native::log("hello0");
        let aa = Closure::new(|| {
            native::log("hello");
        });
        set_interval(&aa, 1).unwrap()
    }

    pub struct TimeoutHandle(TimeoutHandle0, Closure<dyn FnMut()>);
    unsafe impl Send for TimeoutHandle {}

    pub fn spawn_macrotask(mut f: Box<dyn FnMut() + 'static>) -> TimeoutHandle {
        let aa = Closure::new(move || f());
        TimeoutHandle(set_timeout(&aa, 4).unwrap(), aa)
        // TimeoutHandle(Arc::new(Timeout::new(4, move || {
        //     f()
        // })))
    }

    use poll_promise::Promise;
    pub(crate) fn spawn_stuff<T: Send + 'static>(
        f: impl std::future::Future<Output = T> + 'static,
    ) -> poll_promise::Promise<T> {
        Promise::spawn_async(f)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{thread::{spawn, JoinHandle}, sync::{Arc, Mutex}};

    pub struct TimeoutHandle(JoinHandle<()>);
    pub fn spawn_macrotask(f: Box<dyn FnMut() + 'static + Send>) -> TimeoutHandle {
        let spawn = spawn(f);
        TimeoutHandle(spawn)
    }
}
