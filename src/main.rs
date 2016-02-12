extern crate glib_sys;
extern crate winapi;
extern crate libc;

use glib_sys::{gpointer, GThread, g_thread_join, g_thread_new, g_thread_self};
use std::ptr;
use std::thread;
use sys::*;

#[cfg(unix)]
mod sys {
    extern crate libc;

    pub type Native = self::libc::pthread_t;

    pub fn native_self() -> Native {
        unsafe { self::libc::pthread_self() }
    }
}

#[cfg(windows)]
mod sys {
    extern crate kernel32_sys;
    extern crate winapi;

    pub type Native = self::winapi::DWORD;

    pub fn native_self() -> Native {
        unsafe { self::kernel32_sys::GetCurrentThreadId() }
    }
}

#[derive(Clone, Copy, Debug)]
struct Report {
    native: Native,
    gthread: *mut GThread,
}

unsafe impl Send for Report { }

unsafe extern fn report(_: gpointer) -> gpointer {
    let ret = Report {
        native: native_self(),
        gthread: g_thread_self(),
    };
    Box::into_raw(Box::new(ret)) as gpointer
}

fn safe_report() -> Report {
    unsafe { 
        let ret = report(ptr::null_mut()) as *mut Report;
        *Box::from_raw(ret)
    }
}

fn main() {
    let main = safe_report();

    let child = thread::spawn(|| safe_report());

    let glib = unsafe {
        let child = g_thread_new(ptr::null_mut(), Some(report), ptr::null_mut());
        let ret = g_thread_join(child) as *mut Report;
        *Box::from_raw(ret)
    };
    let native = child.join().unwrap();

    assert!(main.native != native.native);
    assert!(main.native != glib.native);
    assert!(native.native != glib.native);
    assert!(main.gthread != native.gthread);
    assert!(main.gthread != glib.gthread);
    assert!(native.gthread != glib.gthread);
}
