use std::process::Command;

use serde_json::Value;
use tench_shared_types::{AcceleratorBackend, AcceleratorDescriptor, HardwareProfile};

pub fn detect_hardware_profile() -> HardwareProfile {
    HardwareProfile {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        accelerators: detect_rocm_accelerators(),
    }
}

fn detect_rocm_accelerators() -> Vec<AcceleratorDescriptor> {
    let output = Command::new("rocm-smi")
        .args([
            "--showdriverversion",
            "--showproductname",
            "--showmeminfo",
            "vram",
            "--showuse",
            "--json",
        ])
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_slice::<Value>(&output.stdout) else {
        return Vec::new();
    };

    parse_rocm_smi_json(&value)
}

pub(crate) fn parse_rocm_smi_json(value: &Value) -> Vec<AcceleratorDescriptor> {
    let Some(cards) = value.as_object() else {
        return Vec::new();
    };

    let mut accelerators = Vec::new();
    for (id, card) in cards {
        if !id.starts_with("card") {
            continue;
        }

        let name = card
            .get("Card Series")
            .and_then(Value::as_str)
            .unwrap_or("AMD GPU")
            .to_string();
        let vendor = card
            .get("Card Vendor")
            .and_then(Value::as_str)
            .unwrap_or("Advanced Micro Devices")
            .to_string();

        accelerators.push(AcceleratorDescriptor {
            id: id.clone(),
            name,
            vendor,
            backend: AcceleratorBackend::Rocm,
            gfx_version: card
                .get("GFX Version")
                .and_then(Value::as_str)
                .map(str::to_string),
            total_memory_bytes: parse_u64_field(card, "VRAM Total Memory (B)"),
            used_memory_bytes: parse_u64_field(card, "VRAM Total Used Memory (B)"),
            utilization_percent: parse_u32_field(card, "GPU use (%)"),
        });
    }

    accelerators
}

fn parse_u64_field(value: &Value, key: &str) -> Option<u64> {
    value
        .get(key)
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<u64>().ok())
}

fn parse_u32_field(value: &Value, key: &str) -> Option<u32> {
    value
        .get(key)
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<u32>().ok())
}
