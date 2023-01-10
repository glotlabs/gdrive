use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub enum ChunkSize {
    Approx1,
    Approx2,
    Approx4,
    Approx8,
    Approx16,
    #[default]
    Approx32,
    Approx64,
    Approx128,
    Approx256,
    Approx512,
    Approx1024,
    Approx2048,
    Approx4096,
    Approx8192,
}

impl ChunkSize {
    pub fn in_bytes(&self) -> u64 {
        match self {
            ChunkSize::Approx1 => u64::pow(2, 20),
            ChunkSize::Approx2 => u64::pow(2, 21),
            ChunkSize::Approx4 => u64::pow(2, 22),
            ChunkSize::Approx8 => u64::pow(2, 23),
            ChunkSize::Approx16 => u64::pow(2, 24),
            ChunkSize::Approx32 => u64::pow(2, 25),
            ChunkSize::Approx64 => u64::pow(2, 26),
            ChunkSize::Approx128 => u64::pow(2, 27),
            ChunkSize::Approx256 => u64::pow(2, 28),
            ChunkSize::Approx512 => u64::pow(2, 29),
            ChunkSize::Approx1024 => u64::pow(2, 30),
            ChunkSize::Approx2048 => u64::pow(2, 31),
            ChunkSize::Approx4096 => u64::pow(2, 32),
            ChunkSize::Approx8192 => u64::pow(2, 33),
        }
    }
}

impl FromStr for ChunkSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(ChunkSize::Approx1),
            "2" => Ok(ChunkSize::Approx2),
            "4" => Ok(ChunkSize::Approx4),
            "8" => Ok(ChunkSize::Approx8),
            "16" => Ok(ChunkSize::Approx16),
            "32" => Ok(ChunkSize::Approx32),
            "64" => Ok(ChunkSize::Approx64),
            "128" => Ok(ChunkSize::Approx128),
            "256" => Ok(ChunkSize::Approx256),
            "512" => Ok(ChunkSize::Approx512),
            "1024" => Ok(ChunkSize::Approx1024),
            "2048" => Ok(ChunkSize::Approx2048),
            "4096" => Ok(ChunkSize::Approx4096),
            "8192" => Ok(ChunkSize::Approx8192),
            _ => Err("Not a valid chunk size, must be a power of 2 between 1 and 8192".to_string()),
        }
    }
}

impl Display for ChunkSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkSize::Approx1 => write!(f, "1"),
            ChunkSize::Approx2 => write!(f, "2"),
            ChunkSize::Approx4 => write!(f, "4"),
            ChunkSize::Approx8 => write!(f, "8"),
            ChunkSize::Approx16 => write!(f, "16"),
            ChunkSize::Approx32 => write!(f, "32"),
            ChunkSize::Approx64 => write!(f, "64"),
            ChunkSize::Approx128 => write!(f, "128"),
            ChunkSize::Approx256 => write!(f, "256"),
            ChunkSize::Approx512 => write!(f, "512"),
            ChunkSize::Approx1024 => write!(f, "1024"),
            ChunkSize::Approx2048 => write!(f, "2048"),
            ChunkSize::Approx4096 => write!(f, "4096"),
            ChunkSize::Approx8192 => write!(f, "8192"),
        }
    }
}
