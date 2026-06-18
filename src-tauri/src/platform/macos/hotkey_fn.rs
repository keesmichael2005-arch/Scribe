use std::sync::atomic::{AtomicBool, Ordering};

use crate::platform::{HotkeyBinding, HotkeyEvent};

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use std::ffi::c_void;

    type IOHIDManagerRef = *mut c_void;
    type IOHIDValueRef = *mut c_void;
    type IOHIDElementRef = *mut c_void;
    type IOReturn = i32;
    type IOOptionBits = u32;
    type CFAllocatorRef = *const c_void;
    type CFRunLoopRef = *const c_void;

    extern "C" {
        fn IOHIDManagerCreate(
            allocator: CFAllocatorRef,
            options: IOOptionBits,
        ) -> IOHIDManagerRef;
        fn IOHIDManagerSetDeviceMatching(
            manager: IOHIDManagerRef,
            matching: *const c_void,
        );
        fn IOHIDManagerRegisterInputValueCallback(
            manager: IOHIDManagerRef,
            callback: extern "C" fn(
                context: *mut c_void,
                result: IOReturn,
                sender: *mut c_void,
                value: IOHIDValueRef,
            ),
            context: *mut c_void,
        );
        fn IOHIDManagerOpen(
            manager: IOHIDManagerRef,
            options: IOOptionBits,
        ) -> IOReturn;
        fn IOHIDManagerScheduleWithRunLoop(
            manager: IOHIDManagerRef,
            run_loop: CFRunLoopRef,
            run_loop_mode: *const c_void,
        );
        fn IOHIDElementGetUsagePage(element: IOHIDElementRef) -> u32;
        fn IOHIDElementGetUsage(element: IOHIDElementRef) -> u32;
        fn IOHIDValueGetElement(value: IOHIDValueRef) -> IOHIDElementRef;
        fn IOHIDValueGetIntegerValue(value: IOHIDValueRef) -> i64;
        fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        fn kCFRunLoopDefaultMode() -> *const c_void;
    }

    const K_IO_HID_MANAGER_OPTION_USE_PERSISTENT_PROPERTIES: IOOptionBits = 1;

    fn is_fn_key(usage_page: u32, usage: u32) -> bool {
        matches!(
            (usage_page, usage),
            (0x07, 0x6A)
                | (0x07, 0x65)
                | (0xFF01, 0x03)
                | (0xFF00, 0x03)
        )
    }

    struct FnTapContext {
        tx: tokio::sync::mpsc::Sender<HotkeyEvent>,
        fn_down: AtomicBool,
    }

    extern "C" fn input_value_callback(
        context: *mut c_void,
        _result: IOReturn,
        _sender: *mut c_void,
        value: IOHIDValueRef,
    ) {
        if context.is_null() {
            return;
        }
        let ctx = unsafe { &*(context as *const FnTapContext) };
        let element = unsafe { IOHIDValueGetElement(value) };
        if element.is_null() {
            return;
        }
        let usage_page = unsafe { IOHIDElementGetUsagePage(element) };
        let usage = unsafe { IOHIDElementGetUsage(element) };
        if !is_fn_key(usage_page, usage) {
            return;
        }
        let int_value = unsafe { IOHIDValueGetIntegerValue(value) };

        let new_down = int_value != 0;
        let was_down = ctx.fn_down.swap(new_down, Ordering::SeqCst);

        if new_down && !was_down {
            tracing::info!(binding = %HotkeyBinding::Fn, event = ?HotkeyEvent::Press);
            let _ = ctx.tx.try_send(HotkeyEvent::Press);
        } else if !new_down && was_down {
            tracing::info!(binding = %HotkeyBinding::Fn, event = ?HotkeyEvent::Release);
            let _ = ctx.tx.try_send(HotkeyEvent::Release);
        }
    }

    pub fn start(
        tx: tokio::sync::mpsc::Sender<HotkeyEvent>,
    ) -> std::io::Result<()> {
        std::thread::Builder::new()
            .name("scribe-fn-tap".into())
            .spawn(move || {
                let manager = unsafe {
                    IOHIDManagerCreate(
                        std::ptr::null(),
                        K_IO_HID_MANAGER_OPTION_USE_PERSISTENT_PROPERTIES,
                    )
                };
                if manager.is_null() {
                    tracing::error!("IOHIDManagerCreate returned null");
                    return;
                }

                unsafe {
                    IOHIDManagerSetDeviceMatching(manager, std::ptr::null());
                }

                let ctx = Box::new(FnTapContext {
                    tx,
                    fn_down: AtomicBool::new(false),
                });
                let ctx_ptr = Box::into_raw(ctx);

                unsafe {
                    IOHIDManagerRegisterInputValueCallback(
                        manager,
                        input_value_callback,
                        ctx_ptr as *mut c_void,
                    );

                    let result = IOHIDManagerOpen(manager, 0);
                    if result != 0 {
                        tracing::error!(result, "IOHIDManagerOpen failed");
                        drop(Box::from_raw(ctx_ptr));
                        return;
                    }

                    let run_loop = CFRunLoopGetCurrent();
                    let mode = kCFRunLoopDefaultMode();
                    IOHIDManagerScheduleWithRunLoop(manager, run_loop, mode);

                    tracing::info!("fn hotkey tap running");

                    #[allow(deprecated)]
                    CFRunLoopRun();
                }
            })?;
        Ok(())
    }

    #[allow(deprecated)]
    extern "C" {
        fn CFRunLoopRun();
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::*;

    pub fn start(
        _tx: tokio::sync::mpsc::Sender<HotkeyEvent>,
    ) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn start(tx: tokio::sync::mpsc::Sender<HotkeyEvent>) -> std::io::Result<()> {
    imp::start(tx)
}
