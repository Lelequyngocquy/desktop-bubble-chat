#[allow(unused_imports)]
use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NSPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NSSize {
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NSRect {
    pub origin: NSPoint,
    pub size: NSSize,
}

impl NSRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: NSPoint { x, y },
            size: NSSize { width, height },
        }
    }
}

pub const K_CG_BITMAP_BYTE_ORDER_32_LITTLE: u32 = 2 << 12;
pub const K_CG_IMAGE_ALPHA_PREMULTIPLIED_FIRST: u32 = 2;
pub const K_CG_RENDERING_INTENT_DEFAULT: u32 = 0;

/// Releases heap-allocated BGRA Vec<u8> when CoreGraphics finishes reading the CGDataProvider
///
/// # Safety
/// `info` must be a valid pointer obtained from `Box::into_raw(Box<Vec<u8>>)` or null.
pub unsafe extern "C" fn release_boxed_bgra_data(info: *mut c_void, _data: *const c_void, _size: usize) {
    if !info.is_null() {
        unsafe {
            let _ = Box::from_raw(info as *mut Vec<u8>);
        }
    }
}

#[cfg(target_os = "macos")]
#[link(name = "objc")]
unsafe extern "C" {
    pub fn objc_autoreleasePoolPush() -> *mut c_void;
    pub fn objc_autoreleasePoolPop(pool: *mut c_void);
}

#[cfg(target_os = "macos")]
#[link(name = "System")]
unsafe extern "C" {
    pub static _dispatch_main_q: c_void;
    pub fn dispatch_async_f(
        queue: *const c_void,
        context: *mut c_void,
        work: unsafe extern "C" fn(*mut c_void),
    );
    pub fn pthread_main_np() -> i32;
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn execute_main_thread_closure(context: *mut c_void) {
    if !context.is_null() {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let task = Box::from_raw(context as *mut Box<dyn FnOnce() + Send>);
            task();
            objc_autoreleasePoolPop(pool);
        }
    }
}

/// Runs a closure inside an Objective-C autoreleasepool on the macOS AppKit main thread
#[cfg(target_os = "macos")]
pub fn run_on_macos_main_thread<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        if pthread_main_np() != 0 {
            let pool = objc_autoreleasePoolPush();
            f();
            objc_autoreleasePoolPop(pool);
        } else {
            let boxed: Box<Box<dyn FnOnce() + Send>> = Box::new(Box::new(f));
            dispatch_async_f(
                &_dispatch_main_q as *const c_void,
                Box::into_raw(boxed) as *mut c_void,
                execute_main_thread_closure,
            );
        }
    }
}

/// Runs a closure synchronously inside an Objective-C autoreleasepool
#[cfg(target_os = "macos")]
pub fn with_autorelease_pool<R, F: FnOnce() -> R>(f: F) -> R {
    unsafe {
        let pool = objc_autoreleasePoolPush();
        let res = f();
        objc_autoreleasePoolPop(pool);
        res
    }
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub fn CGDataProviderCreateWithData(
        info: *mut c_void,
        data: *const c_void,
        size: usize,
        release_data: Option<unsafe extern "C" fn(*mut c_void, *const c_void, usize)>,
    ) -> *mut c_void;
    pub fn CGColorSpaceCreateDeviceRGB() -> *mut c_void;
    pub fn CGColorSpaceRelease(space: *mut c_void);
    pub fn CGDataProviderRelease(provider: *mut c_void);
    pub fn CGImageRelease(image: *mut c_void);
    pub fn CGImageCreate(
        width: usize,
        height: usize,
        bits_per_component: usize,
        bits_per_pixel: usize,
        bytes_per_row: usize,
        space: *mut c_void,
        bitmap_info: u32,
        provider: *mut c_void,
        decode: *const f64,
        should_interpolate: bool,
        intent: u32,
    ) -> *mut c_void;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cg_little_endian_constant_and_release_callback() {
        assert_eq!(K_CG_BITMAP_BYTE_ORDER_32_LITTLE, 2 << 12);
        let boxed = Box::new(vec![1u8, 2, 3, 4]);
        let ptr = boxed.as_ptr() as *const c_void;
        let len = boxed.len();
        let info = Box::into_raw(boxed) as *mut c_void;
        unsafe {
            release_boxed_bgra_data(info, ptr, len);
        }
    }
}
