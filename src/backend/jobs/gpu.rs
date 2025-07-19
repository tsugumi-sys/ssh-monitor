use super::job::JobResult;
use anyhow::Result;
use serde::Serialize;

pub const GPU_COMMAND: &str = r#"bash -c '
if [[ "$(uname)" == "Darwin" ]]; then
  system_profiler SPDisplaysDataType
else
  nvidia-smi --query-gpu=name,memory.total,memory.used,temperature.gpu --format=csv,noheader,nounits 2>&1
fi
'"#;

#[derive(Debug, Serialize, Clone)]
pub struct GpuInfo {
    pub index: u32,
    pub name: String,
    pub memory_total_mb: Option<u64>,
    pub memory_used_mb: Option<u64>,
    pub temperature_c: Option<f32>,
    pub raw_output: Option<String>,
}

fn parse_gpu_linux(output: &str) -> Vec<GpuInfo> {
    let mut infos = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let parts: Vec<&str> = line.split(',').map(|p| p.trim()).collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();
            let memory_total_mb = parts[1].parse::<u64>().ok();
            let memory_used_mb = parts[2].parse::<u64>().ok();
            let temperature_c = parts[3].parse::<f32>().ok();
            infos.push(GpuInfo {
                index: idx as u32,
                name,
                memory_total_mb,
                memory_used_mb,
                temperature_c,
                raw_output: None,
            });
        }
    }
    infos
}

fn parse_gpu_macos(output: &str) -> Vec<GpuInfo> {
    let mut infos = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        if let Some(rest) = line.split_once("Chipset Model:") {
            infos.push(GpuInfo {
                index: idx as u32,
                name: rest.1.trim().to_string(),
                memory_total_mb: None,
                memory_used_mb: None,
                temperature_c: None,
                raw_output: None,
            });
        }
    }
    infos
}

pub fn parse_gpu(output: &str) -> Result<Option<JobResult>> {
    Ok(Some(JobResult {
        job_name: "gpu".into(),
        value: Box::new(if output.contains("Chipset Model:") {
            parse_gpu_macos(output)
        } else {
            parse_gpu_linux(output)
        }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test for Linux GPU parsing
    #[test]
    fn test_parse_gpu_linux() -> Result<()> {
        // Arrange
        let input = "GeForce GTX 1080 Ti, 11178, 4523, 70\nTesla K80, 11441, 0, 35";

        // Act
        let result = parse_gpu(input)?;

        // Assert
        assert!(result.is_some());
        let job_result = result.unwrap();
        let infos: &Vec<GpuInfo> = job_result.value.downcast_ref::<Vec<GpuInfo>>().unwrap();
        assert_eq!(infos.len(), 2);

        // Check first GPU info
        assert_eq!(infos[0].name, "GeForce GTX 1080 Ti");
        assert_eq!(infos[0].memory_total_mb, Some(11178));
        assert_eq!(infos[0].memory_used_mb, Some(4523));
        assert_eq!(infos[0].temperature_c, Some(70.0));

        // Check second GPU info
        assert_eq!(infos[1].name, "Tesla K80");
        assert_eq!(infos[1].memory_total_mb, Some(11441));
        assert_eq!(infos[1].memory_used_mb, Some(0));
        assert_eq!(infos[1].temperature_c, Some(35.0));

        Ok(())
    }

    // Test for macOS GPU parsing
    #[test]
    fn test_parse_gpu_macos() -> Result<()> {
        // Arrange
        let macos_input = r#"
Graphics/Displays:

    Apple M3 Pro:

      Chipset Model: Apple M3 Pro
      Type: GPU
      Bus: Built-In
      Total Number of Cores: 14
      Vendor: Apple (0x106b)
      Metal Support: Metal 3
      Displays:
        Color LCD:
          Display Type: Built-in Liquid Retina XDR Display
          Resolution: 3024 x 1964 Retina
          Main Display: Yes
          Mirror: Off
          Online: Yes
          Automatically Adjust Brightness: No
          Connection Type: Internal
"#;

        // Act
        let result = parse_gpu(macos_input)?;

        // Assert
        assert!(result.is_some());
        let job_result = result.unwrap();
        let infos: &Vec<GpuInfo> = job_result.value.downcast_ref::<Vec<GpuInfo>>().unwrap();
        assert_eq!(infos.len(), 1);

        // Check macOS GPU info
        assert_eq!(infos[0].name, "Apple M3 Pro");
        assert_eq!(infos[0].memory_total_mb, None);
        assert_eq!(infos[0].memory_used_mb, None);
        assert_eq!(infos[0].temperature_c, None);

        Ok(())
    }
}
