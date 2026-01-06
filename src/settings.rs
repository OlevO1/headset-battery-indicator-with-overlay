use anyhow::{Context, Result};
use winreg::enums::HKEY_CURRENT_USER;

#[derive(Debug, Clone)]
pub struct Settings {
    pub notifications_enabled: bool,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey("Software\\HeadsetBatteryIndicator")
            .context("accessing registry key")?;

        let notifications_enabled: u32 = key.get_value("NotificationsEnabled").unwrap_or_default();

        log::debug!(
            "NotificationsEnabled={}",
            notifications_enabled
        );

        Ok(Self {
            notifications_enabled: notifications_enabled != 0,
        })
    }

    pub fn save(&self) -> Result<()> {
        let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey("Software\\HeadsetBatteryIndicator")
            .context("accessing registry key")?;

        key.set_value("NotificationsEnabled", &(self.notifications_enabled as u32))
            .context("setting NotificationsEnabled value")?;

        log::debug!(
            "Set NotificationsEnabled={}",
            self.notifications_enabled
        );

        Ok(())
    }
}
