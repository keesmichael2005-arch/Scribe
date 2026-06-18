use std::time::Duration;

use crate::platform::{PermissionKind, PermissionStatus};
use objc2::msg_send;
use objc2::rc::Retained;
use objc2_foundation::NSString;

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
    use std::sync::mpsc;
    use objc2::runtime::Bool;

    let (tx, rx) = mpsc::channel();

    let stack = block2::StackBlock::new(move |granted: Bool| {
        let status = if granted.as_bool() {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Denied
        };
        let _ = tx.send(status);
    });
    let block = stack.copy();
    let cls = objc2::class!(AVCaptureDevice);
    let audio = av_media_type_audio();
    unsafe {
        let _: () = msg_send![
            cls,
            requestAccessForMediaType: &*audio,
            completionHandler: &*block
        ];
    }
    rx.recv().unwrap_or(PermissionStatus::Denied)
}

#[cfg(not(target_os = "macos"))]
pub async fn request_microphone_permission() -> PermissionStatus {
    PermissionStatus::NotDetermined
}

pub fn poll_status(
    kind: PermissionKind,
    tx: tokio::sync::mpsc::Sender<PermissionStatus>,
) {
    tokio::spawn(async move {
        loop {
            let status = match kind {
                PermissionKind::Microphone => microphone_status(),
                PermissionKind::Accessibility => accessibility_status(),
            };
            if tx.send(status).await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
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
