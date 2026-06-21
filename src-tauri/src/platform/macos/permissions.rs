use std::time::Duration;

use crate::platform::PermissionStatus;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2_foundation::NSString;

#[link(name = "AVFoundation", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

fn av_media_type_audio() -> Retained<NSString> {
    NSString::from_str("soun")
}

#[cfg(target_os = "macos")]
fn microphone_status_impl() -> PermissionStatus {
    let cls = objc2::class!(AVCaptureDevice);
    let audio = av_media_type_audio();
    unsafe {
        let status: isize = msg_send![
            cls,
            authorizationStatusForMediaType: &*audio
        ];
        match status {
            0 => PermissionStatus::NotDetermined,
            1 => PermissionStatus::Denied,
            2 => PermissionStatus::Denied,
            3 => PermissionStatus::Granted,
            _ => PermissionStatus::NotDetermined,
        }
    }
}

#[cfg(target_os = "macos")]
fn accessibility_status_impl() -> PermissionStatus {
    unsafe {
        if AXIsProcessTrusted() {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Denied
        }
    }
}

pub fn microphone_status() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    return microphone_status_impl();
    #[allow(unreachable_code)]
    PermissionStatus::NotDetermined
}

pub fn accessibility_status() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    return accessibility_status_impl();
    #[allow(unreachable_code)]
    PermissionStatus::NotDetermined
}

#[cfg(target_os = "macos")]
pub async fn request_microphone_permission() -> PermissionStatus {
    use block2::RcBlock;
    use dispatch2::DispatchQueue;
    use objc2::runtime::Bool;
    use tokio::sync::oneshot;

    let (tx, rx) = oneshot::channel();

    DispatchQueue::main().exec_async(move || {
        use std::cell::RefCell;

        let tx = RefCell::new(Some(tx));
        let block = RcBlock::new(move |granted: Bool| {
            let status = if granted.as_bool() {
                PermissionStatus::Granted
            } else {
                PermissionStatus::Denied
            };
            if let Some(tx) = tx.borrow_mut().take() {
                let _ = tx.send(status);
            }
        });
        let cls = objc2::class!(AVCaptureDevice);
        let audio = av_media_type_audio();
        unsafe {
            let _: () = msg_send![
                cls,
                requestAccessForMediaType: &*audio,
                completionHandler: &*block
            ];
        }
    });

    match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(status)) => status,
        Ok(Err(_)) => PermissionStatus::Denied,
        Err(_) => {
            tracing::warn!("microphone permission request timed out");
            microphone_status()
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub async fn request_microphone_permission() -> PermissionStatus {
    PermissionStatus::NotDetermined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn microphone_status_returns_variant() {
        let status = microphone_status();
        assert!(
            matches!(
                status,
                PermissionStatus::Granted
                    | PermissionStatus::Denied
                    | PermissionStatus::NotDetermined
            ),
            "expected one of the three PermissionStatus variants"
        );
    }

    #[test]
    fn accessibility_status_returns_variant() {
        let status = accessibility_status();
        assert!(
            matches!(
                status,
                PermissionStatus::Granted
                    | PermissionStatus::Denied
                    | PermissionStatus::NotDetermined
            ),
            "expected one of the three PermissionStatus variants"
        );
    }
}
