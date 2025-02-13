//! Collection of item specs the peace framework.
//!
//! Every item spec crate needs to be enabled with its own feature. Example:
//!
//! ```toml
//! peace_item_specs = { version = "0.0.3", features = ["file_download"] }
//! ```
//!
//! In code:
//!
//! ```rust
//! #[cfg(feature = "file_download")]
//! # fn main() {
//! use peace::cfg::{item_spec_id, ItemSpecId};
//! use peace_item_specs::file_download::FileDownloadItemSpec;
//!
//! /// Marker type for `FileDownloadParams`.
//! #[derive(Clone, Copy, Debug, PartialEq, Eq)]
//! pub struct MyFileId;
//!
//! let file_download_item_spec =
//!     FileDownloadItemSpec::<MyFileId>::new(item_spec_id!("file_to_download"));
//! # }
//! #
//! #[cfg(not(feature = "file_download"))]
//! # fn main() {}
//! ```

// Re-exports
#[cfg(feature = "blank")]
pub use peace_item_spec_blank as blank;
#[cfg(feature = "file_download")]
pub use peace_item_spec_file_download as file_download;
#[cfg(feature = "sh_cmd")]
pub use peace_item_spec_sh_cmd as sh_cmd;
#[cfg(feature = "sh_sync_cmd")]
pub use peace_item_spec_sh_sync_cmd as sh_sync_cmd;
#[cfg(feature = "tar_x")]
pub use peace_item_spec_tar_x as tar_x;
