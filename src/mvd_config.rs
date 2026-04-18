use serde::Deserialize;
use std::fs;
use std::sync::OnceLock;

/// Configuration loaded from `~/.mvd`
#[derive(Debug, Deserialize, Default)]
pub struct UserConfig {
    pub ram_usage_ratio: Option<f64>,
    pub storage_usage_ratio: Option<f64>,
}

impl UserConfig {
    pub fn load() -> Option<Self> {
        let home = dirs_next::home_dir()?;
        let config_path = home.join(".mvd");

        let content = fs::read_to_string(config_path).ok()?;
        serde_json::from_str(&content).ok()
    }
}

pub(crate) fn dynamic_capacity() -> u64 {
    static CAPACITY: OnceLock<u64> = OnceLock::new();

    *CAPACITY.get_or_init(|| {
        let config = UserConfig::load().unwrap_or_default();
        let ram_ratio = config.ram_usage_ratio.unwrap_or(0.5);
        let storage_ratio = config.storage_usage_ratio.unwrap_or(0.2);

        let mut sys = sysinfo::System::new();
        // Refresh memory to get total RAM
        sys.refresh_memory();

        let disks = sysinfo::Disks::new_with_refreshed_list();

        let total_ram_bytes = sys.total_memory();

        let available_disk_bytes = disks
            .iter()
            .find(|d| d.mount_point() == std::path::Path::new("/"))
            .map(|d| d.available_space())
            .unwrap_or_else(|| {
                // fallback to the largest available disk
                disks
                    .iter()
                    .map(|d| d.available_space())
                    .max()
                    .unwrap_or(500 * 1024 * 1024 * 1024) // 500 GB fallback
            });

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let max_ram_usage = (total_ram_bytes as f64 * ram_ratio) as u64;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let max_storage_usage = (available_disk_bytes as f64 * storage_ratio) as u64;

        let limit = std::cmp::min(max_ram_usage, max_storage_usage);

        // Safety guard: ensure the limit is never utterly tiny if ratios are misconfigured
        // (minimum 50MB) 
        std::cmp::max(limit, 50 * 1024 * 1024)
    })
}
