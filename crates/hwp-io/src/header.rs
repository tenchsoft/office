use crate::error::HwpError;

/// HWP file version (major.minor.micro.build)
#[derive(Debug, Clone)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
    pub build: u8,
}

impl Version {
    pub fn from_u32(v: u32) -> Self {
        Self {
            major: ((v >> 24) & 0xFF) as u8,
            minor: ((v >> 16) & 0xFF) as u8,
            micro: ((v >> 8) & 0xFF) as u8,
            build: (v & 0xFF) as u8,
        }
    }

    pub fn as_u32(&self) -> u32 {
        ((self.major as u32) << 24)
            | ((self.minor as u32) << 16)
            | ((self.micro as u32) << 8)
            | (self.build as u32)
    }

    pub fn is_at_least(&self, major: u8, minor: u8, micro: u8, build: u8) -> bool {
        self.as_u32()
            >= ((major as u32) << 24 | (minor as u32) << 16 | (micro as u32) << 8 | build as u32)
    }
}

/// HWP file header (256 bytes)
#[derive(Debug, Clone)]
pub struct FileHeader {
    pub signature: [u8; 32],
    pub version: Version,
    pub flags: u32,
    pub flags2: u32,
    pub encrypt_version: u32,
    pub kogl: u8,
}

impl FileHeader {
    pub fn parse(data: &[u8]) -> Result<Self, HwpError> {
        if data.len() < 256 {
            return Err(HwpError::Header("FileHeader too short".into()));
        }

        // Verify signature: "HWP Document File\0" padded to 32 bytes
        let expected_sig = b"HWP Document File\0";
        if !data[..18].eq_ignore_ascii_case(expected_sig) {
            return Err(HwpError::Header("Invalid HWP signature".into()));
        }

        let version =
            Version::from_u32(u32::from_le_bytes([data[32], data[33], data[34], data[35]]));
        let flags = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);
        let flags2 = u32::from_le_bytes([data[40], data[41], data[42], data[43]]);
        let encrypt_version = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);
        let kogl = data[48];

        let mut signature = [0u8; 32];
        signature.copy_from_slice(&data[..32]);

        Ok(Self {
            signature,
            version,
            flags,
            flags2,
            encrypt_version,
            kogl,
        })
    }

    pub fn is_compressed(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    pub fn is_encrypted(&self) -> bool {
        (self.flags & 0x02) != 0
    }

    pub fn is_distributable(&self) -> bool {
        (self.flags & 0x04) != 0
    }

    pub fn has_script(&self) -> bool {
        (self.flags & 0x08) != 0
    }

    pub fn has_drm(&self) -> bool {
        (self.flags & 0x10) != 0
    }

    pub fn has_track_changes(&self) -> bool {
        (self.flags & (1 << 14)) != 0
    }

    pub fn to_bytes(&self) -> [u8; 256] {
        let mut buf = [0u8; 256];
        buf[..32].copy_from_slice(&self.signature);
        buf[32..36].copy_from_slice(&self.version.as_u32().to_le_bytes());
        buf[36..40].copy_from_slice(&self.flags.to_le_bytes());
        buf[40..44].copy_from_slice(&self.flags2.to_le_bytes());
        buf[44..48].copy_from_slice(&self.encrypt_version.to_le_bytes());
        buf[48] = self.kogl;
        buf
    }

    pub fn default_v5() -> Self {
        let mut signature = [0u8; 32];
        signature[..18].copy_from_slice(b"HWP Document File\0");
        Self {
            signature,
            version: Version::from_u32(0x05010000), // 5.1.0.0
            flags: 0x01,                            // compressed
            flags2: 0,
            encrypt_version: 0,
            kogl: 0,
        }
    }
}
