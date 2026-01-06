use crate::headset_control::BatteryState;

use win32_notif::{NotificationBuilder, ToastsNotifier, notification::visual::{Image, Placement, Text, image::ImageCrop, text::HintStyle}};
#[cfg(windows)]
use windows::{
    Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID,
    Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
        IPersistFile,
    },
    Win32::UI::Shell::PropertiesSystem::IPropertyStore,
    Win32::UI::Shell::{IShellLinkW, SetCurrentProcessExplicitAppUserModelID, ShellLink},
    core::{HSTRING, PCWSTR},
};

pub struct Notifier {
    toast_notifier: ToastsNotifier,
    last_notification_state: Option<(isize, BatteryState)>,
}

impl Notifier {
    pub fn new() -> anyhow::Result<Self> {
        const TOAST_APP_ID: &str = "HeadsetBatteryIndicator.App";

        let app_id = register_toast_app(TOAST_APP_ID)
            .map(|_| TOAST_APP_ID)
            .unwrap_or_else(|err| {
                log::error!("Toast registration failed; falling back to Explorer AUMID: {err:?}");
                "Microsoft.Windows.Explorer"
            });

        let toast_notifier = ToastsNotifier::new(app_id)?;

        Ok(Self {
            toast_notifier,
            last_notification_state: None,
        })
    }

    pub fn update_notifier(
        &mut self,
        current_level: isize,
        current_status: BatteryState,
        product_name: &str,
    ) {
        if let Some((last_level, last_status)) = self.last_notification_state {
            let mut msg = None;

            let battery_discharging = current_status == BatteryState::BatteryAvailable;
            let battery_charging = current_status == BatteryState::BatteryCharging;
            
            // Low battery (10%)
            if current_level <= 10
                && last_level > 10
                && battery_discharging
            {
                msg = Some(format!("Battery low ({}%)", current_level));
            }
            // Critical battery (3%)
            else if current_level <= 3
                && last_level > 3
                && battery_discharging
            {
                msg = Some(format!("Battery critical ({}%)", current_level));
            }
            // Charging started
            else if battery_charging
                && last_status != BatteryState::BatteryCharging
            {
                msg = Some(format!("Charging started ({}%)", current_level));
            }
            // Battery full (100%)
            else if current_level == 100
                && last_level < 100
                && battery_charging
            {
                msg = Some("Battery full".to_string());
            }

            if let Some(body) = msg {
                self.show_notification(current_level, current_status, product_name, body);
            }
        }

        self.last_notification_state = Some((current_level, current_status));
    }

    fn show_notification(
        &mut self,
        current_level: isize,
        current_status: BatteryState,
        product_name: &str,
        body: String,
    ) {
        let mut builder = NotificationBuilder::new()
            .visual(Text::create(0, product_name).with_style(HintStyle::Title))
            .visual(Text::create(1, &body).with_style(HintStyle::Body));

        if let Some(logo_uri) = toast_notif_logo_uri(current_level, current_status) {
            builder = builder.visual(
                Image::create(2, &logo_uri)
                    .with_placement(Placement::AppLogoOverride)
                    .with_crop(ImageCrop::None),
            );
        }

        match builder.build(
            current_level as u32,
            &self.toast_notifier,
            &format!("battery_{}", current_level),
            "battery",
        ) {
            Ok(notif) => {
                if let Err(e) = notif.show() {
                    log::error!("Failed to show notification: {e:?}");
                }
            }
            Err(e) => {
                log::error!("Failed to build notification: {e:?}");
            }
        }
    }

    pub fn send_test_notification(&mut self) {
        self.show_notification(50, BatteryState::BatteryAvailable, "Test Device", "Battery critical (50%)".to_string());
    }
}

#[cfg(windows)]
use windows::core::{Interface, PROPVARIANT};

pub fn embedded_notif_png(
    battery_percent: isize,
    charging: bool,
) -> Option<(&'static [u8], &'static str)> {
    // Notification icon set in src/icons/notifs:
    // batt-5/10/25/50/75/full, with optional -charg.
    let bucket = match battery_percent {
        0..=7 => "5",
        8..=17 => "10",
        18..=37 => "25",
        38..=62 => "50",
        63..=87 => "75",
        _ => "full",
    };

    let key = match (bucket, charging) {
        ("5", false) => (
            include_bytes!("icons/notifs/batt-5.png").as_slice(),
            "batt-5.png",
        ),
        ("5", true) => (
            include_bytes!("icons/notifs/batt-5-charg.png").as_slice(),
            "batt-5-charg.png",
        ),
        ("10", false) => (
            include_bytes!("icons/notifs/batt-10.png").as_slice(),
            "batt-10.png",
        ),
        ("10", true) => (
            include_bytes!("icons/notifs/batt-10-charg.png").as_slice(),
            "batt-10-charg.png",
        ),
        ("25", false) => (
            include_bytes!("icons/notifs/batt-25.png").as_slice(),
            "batt-25.png",
        ),
        ("25", true) => (
            include_bytes!("icons/notifs/batt-25-charg.png").as_slice(),
            "batt-25-charg.png",
        ),
        ("50", false) => (
            include_bytes!("icons/notifs/batt-50.png").as_slice(),
            "batt-50.png",
        ),
        ("50", true) => (
            include_bytes!("icons/notifs/batt-50-charg.png").as_slice(),
            "batt-50-charg.png",
        ),
        ("75", false) => (
            include_bytes!("icons/notifs/batt-75.png").as_slice(),
            "batt-75.png",
        ),
        ("75", true) => (
            include_bytes!("icons/notifs/batt-75-charg.png").as_slice(),
            "batt-75-charg.png",
        ),
        ("full", false) => (
            include_bytes!("icons/notifs/batt-full.png").as_slice(),
            "batt-full.png",
        ),
        ("full", true) => (
            include_bytes!("icons/notifs/batt-full-charg.png").as_slice(),
            "batt-full-charg.png",
        ),
        _ => return None,
    };

    Some(key)
}

pub fn toast_notif_logo_uri(battery_percent: isize, state: BatteryState) -> Option<String> {
    let charging = state == BatteryState::BatteryCharging;
    let (png_bytes, filename) = embedded_notif_png(battery_percent, charging)?;

    let dir = toast_cache_dir()?;

    // App logo override must be square; generate a square version of the wide 113x51 PNG.
    let logo_name = format!("logo-{filename}");
    let logo_path = dir.join(logo_name);
    if !logo_path.exists() {
        let decoded = match image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png)
        {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                log::error!("Failed to decode notif png for toast logo: {e:?}");
                return None;
            }
        };

        let (w, h) = decoded.dimensions();
        let side = w.max(h);
        let mut square = image::RgbaImage::from_pixel(side, side, image::Rgba([0, 0, 0, 0]));
        let x = (side - w) / 2;
        let y = (side - h) / 2;

        for yy in 0..h {
            for xx in 0..w {
                let p = *decoded.get_pixel(xx, yy);
                square.put_pixel(x + xx, y + yy, p);
            }
        }

        if let Err(e) = image::DynamicImage::ImageRgba8(square)
            .save_with_format(&logo_path, image::ImageFormat::Png)
        {
            log::error!("Failed to write toast logo png to {:?}: {e:?}", logo_path);
            return None;
        }
    }

    path_to_file_uri(&logo_path)
}

#[cfg(windows)]
pub fn register_toast_app(app_id: &str) -> anyhow::Result<()> {
    // Win32 Toast notifications typically require a Start Menu shortcut whose
    // AppUserModelID matches the notifier ID. Without this, `show()` can succeed
    // but nothing appears.

    // Ensure the system associates this running EXE with the same AUMID.
    unsafe {
        use anyhow::Context;

        SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(app_id))
            .context("SetCurrentProcessExplicitAppUserModelID")
    }
}

fn toast_cache_dir() -> Option<std::path::PathBuf> {
    // Use LocalAppData instead of Roaming config dir.
    // Toast image loading is more reliable from LocalAppData for unpackaged apps.
    let mut dir = dirs::data_local_dir().unwrap_or_else(|| std::env::temp_dir());
    dir.push("headset-battery-indicator");
    dir.push("toast-icons");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn path_to_file_uri(path: &std::path::Path) -> Option<String> {
    // Avoid `canonicalize()` here: on Windows it often returns a `\\?\C:\...` path
    // which produces a broken `file://///?/C:/...` URI and the toast image silently fails.
    let mut s = path.to_string_lossy().to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        s = stripped.to_string();
    }
    s = s.replace('\\', "/");

    // Drive-letter absolute path
    if s.len() >= 2 && s.as_bytes().get(1) == Some(&b':') {
        return Some(format!("file:///{s}"));
    }

    // Fallback: treat as already-rooted
    if s.starts_with('/') {
        return Some(format!("file://{s}"));
    }

    None
}

#[cfg(windows)]
fn to_wide_null_terminated(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    s.encode_wide().chain(std::iter::once(0)).collect()
}
