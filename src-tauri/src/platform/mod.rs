pub mod version;

pub use version::{get_windows_version, is_windows_build_at_least};

#[link(name = "winmm")]
extern "system" {
    pub fn timeBeginPeriod(uPeriod: u32) -> u32;
}