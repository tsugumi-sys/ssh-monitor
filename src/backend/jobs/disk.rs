use super::job::JobResult;
use anyhow::Result;
use serde::Serialize;

pub const DISK_COMMAND: &str = "df -Pm | tail -n +2";

#[derive(Debug, Serialize, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub used_percent: f32,
}

pub fn parse_disk(output: &str) -> Result<Option<JobResult>> {
    let mut results = vec![];

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let total_mb = parts[1].parse::<u64>().unwrap_or(0);
        let used_mb = parts[2].parse::<u64>().unwrap_or(0);
        let available_mb = parts[3].parse::<u64>().unwrap_or(0);
        let used_percent = parts[4].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
        let mount_point = parts[5].to_string();

        results.push(DiskInfo {
            mount_point,
            total_mb,
            used_mb,
            available_mb,
            used_percent,
        });
    }

    Ok(Some(JobResult {
        job_name: "disk".into(),
        value: Box::new(results),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sample input data (df -h command output)
    const SAMPLE_OUTPUT: &str = r#"
/dev/disk3s1s1    471482  15125     48654    24%    /
devfs                  0      0         0   100%    /dev
/dev/disk3s6      471482  10240     48654    18%    /System/Volumes/VM
/dev/disk3s2      471482  13278     48654    22%    /System/Volumes/Preboot
/dev/disk3s4      471482    685     48654     2%    /System/Volumes/Update
/dev/disk1s2         500      6       481     2%    /System/Volumes/xarts
/dev/disk1s1         500      5       481     2%    /System/Volumes/iSCPreboot
/dev/disk1s3         500      2       481     1%    /System/Volumes/Hardware
/dev/disk3s5      471482 381304     48654    89%    /System/Volumes/Data
map auto_home          0      0         0   100%    /System/Volumes/Data/home
/dev/disk5s1        8578   8303       248    98%    /Library/Developer/CoreSimulator/Cryptex/Images/bundle/SimRuntimeBundle-CB22B552-AEC8-42E4-9003-0C2827873D1F
/dev/disk7s1       18878  18358       472    98%    /Library/Developer/CoreSimulator/Volumes/iOS_22C150
/dev/disk3s3      471482   2040     48654     5%    /Volumes/Recovery
/dev/disk2s1        5119   1888      3210    38%    /System/Volumes/Update/SFR/mnt1
/dev/disk3s1      471482  15125     48654    24%    /System/Volumes/Update/mnt1
"#;

    #[test]
    fn test_parse_disk() {
        let result = parse_disk(SAMPLE_OUTPUT);
        assert!(result.is_ok());

        // Unwrap the Option<JobResult> to get the JobResult
        let job_result = result.unwrap().expect("Job result should be present");

        assert_eq!(job_result.job_name, "disk");

        // Clone the value from the JobResult (downcast_ref and then clone)
        let disk_info_list: Vec<DiskInfo> =
            (*job_result.value.downcast_ref::<Vec<DiskInfo>>().unwrap()).clone();

        // Validate the first entry
        assert_eq!(disk_info_list[0].mount_point, "/");
        assert_eq!(disk_info_list[0].total_mb, 471482);
        assert_eq!(disk_info_list[0].used_mb, 15125);
        assert_eq!(disk_info_list[0].available_mb, 48654);
        assert_eq!(disk_info_list[0].used_percent, 24.0);

        // Validate the last entry
        assert_eq!(
            disk_info_list[disk_info_list.len() - 1].mount_point,
            "/System/Volumes/Update/mnt1"
        );
        assert_eq!(disk_info_list[disk_info_list.len() - 1].total_mb, 471482);
        assert_eq!(disk_info_list[disk_info_list.len() - 1].used_mb, 15125);
        assert_eq!(disk_info_list[disk_info_list.len() - 1].available_mb, 48654);
        assert_eq!(disk_info_list[disk_info_list.len() - 1].used_percent, 24.0);
    }
}
