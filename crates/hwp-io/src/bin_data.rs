//! Binary data (images) extracted from HWP file.

/// Extract embedded images from BinData entries.
#[allow(dead_code)]
pub fn extract_images(bin_data: &[(String, Vec<u8>)]) -> Vec<(String, Vec<u8>)> {
    bin_data.to_vec()
}
