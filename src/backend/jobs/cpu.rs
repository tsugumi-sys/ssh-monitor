use super::job::JobResult;
use anyhow::Result;
use serde::Serialize;

pub const CPU_COMMAND: &str = r#"bash -c '
  if [[ "$(uname)" == "Darwin" ]]; then
    sysctl -a | grep machdep.cpu && echo __LSTCPU__ && ps -A -o %cpu && echo __PSCPU__ && ps aux | awk "NR>1 {sum+=$3} END {print sum}" && echo __MPSTAT__;
  else
    lscpu && echo __STAT__ && cat /proc/stat | grep "cpu" | awk '\''{usage=($2+$3+$4+$6+$7)*100/($2+$3+$4+$5+$6+$7+$8+$9+$10); print $1, usage}'\'' && echo __PSCPU__;
  fi
'"#;

#[derive(Debug, Serialize, Clone)]
pub struct CpuInfo {
    pub model_name: String,
    pub core_count: usize,
    pub usage_percent: f32,
    pub per_core: Vec<f32>,
}

pub fn parse_cpu(output: &str) -> Result<Option<JobResult>> {
    let parts: Vec<&str> = output.split("__STAT__").collect();
    if parts.len() != 2 {
        return Ok(None);
    }

    let (info_part, stat_part) = (parts[0], parts[1]);
    let is_mac = info_part.contains("machdep.cpu");

    let (model_name, core_count) = if is_mac {
        let name = info_part
            .lines()
            .find(|l| l.contains("machdep.cpu.brand_string"))
            .and_then(|l| l.split(':').nth(1))
            .unwrap_or("")
            .trim()
            .to_string();

        let cores = info_part
            .lines()
            .find(|l| l.contains("machdep.cpu.core_count"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1);
        (name, cores)
    } else {
        let name = info_part
            .lines()
            .find(|l| l.contains("Model name"))
            .or_else(|| info_part.lines().find(|l| l.contains("model name")))
            .and_then(|l| l.split(':').nth(1))
            .unwrap_or("")
            .trim()
            .to_string();

        let cores = info_part
            .lines()
            .find(|l| l.contains("CPU(s):"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1);
        (name, cores)
    };

    let (usage_percent, per_core) = if is_mac {
        let mut sum = 0f32;
        let mut core_usages = vec![];

        for line in stat_part.lines() {
            if let Ok(p) = line.trim().parse::<f32>() {
                core_usages.push(p);
                sum += p;
            }
        }

        (sum, core_usages)
    } else {
        let mut per_core_usages = vec![];

        for line in stat_part.lines() {
            if line.starts_with("cpu") && !line.contains("cpu ") {
                let cpu_usage_data: Vec<&str> = line.split_whitespace().collect();
                if cpu_usage_data.len() > 1 {
                    if let Ok(usage) = cpu_usage_data[1].trim_end_matches('%').parse::<f32>() {
                        per_core_usages.push(usage);
                    }
                }
            }
        }

        let usage_percent = if !per_core_usages.is_empty() {
            per_core_usages.iter().sum::<f32>() / per_core_usages.len() as f32
        } else {
            0.0
        };

        (usage_percent, per_core_usages)
    };

    let info = CpuInfo {
        model_name,
        core_count,
        usage_percent,
        per_core,
    };

    Ok(Some(JobResult {
        job_name: "cpu".into(),
        value: Box::new(info),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_cpu() -> Result<()> {
        let input = r#"
__BEGIN_cpu__
Architecture:                         x86_64
CPU op-mode(s):                       32-bit, 64-bit
Address sizes:                        48 bits physical, 48 bits virtual
Byte Order:                           Little Endian
CPU(s):                               32
On-line CPU(s) list:                  0-31
Vendor ID:                            AuthenticAMD
Model name:                           AMD Ryzen 9 5950X 16-Core Processor
CPU family:                           25
Model:                                33
Thread(s) per core:                   2
Core(s) per socket:                   16
Socket(s):                            1
Stepping:                             0
Frequency boost:                      enabled
CPU max MHz:                          3400.0000
CPU min MHz:                          2200.0000
BogoMIPS:                             6800.15
Flags:                                fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant_tsc rep_good nopl nonstop_tsc cpuid extd_apicid aperfmperf rapl pni pclmulqdq monitor ssse3 fma cx16 sse4_1 sse4_2 x2apic movbe popcnt aes xsave avx f16c rdrand lahf_lm cmp_legacy svm extapic cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw ibs skinit wdt tce topoext perfctr_core perfctr_nb bpext perfctr_llc mwaitx cpb cat_l3 cdp_l3 hw_pstate ssbd mba ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 erms invpcid cqm rdt_a rdseed adx smap clflushopt clwb sha_ni xsaveopt xsavec xgetbv1 xsaves cqm_llc cqm_occup_llc cqm_mbm_total cqm_mbm_local clzero irperf xsaveerptr rdpru wbnoinvd arat npt lbrv svm_lock nrip_save tsc_scale vmcb_clean flushbyasid decodeassists pausefilter pfthreshold avic v_vmsave_vmload vgif v_spec_ctrl umip pku ospke vaes vpclmulqdq rdpid overflow_recov succor smca fsrm
Virtualization:                       AMD-V
L1d cache:                            512 KiB (16 instances)
L1i cache:                            512 KiB (16 instances)
L2 cache:                             8 MiB (16 instances)
L3 cache:                             64 MiB (2 instances)
NUMA node(s):                         1
NUMA node0 CPU(s):                    0-31
Vulnerability Gather data sampling:   Not affected
Vulnerability Itlb multihit:          Not affected
Vulnerability L1tf:                   Not affected
Vulnerability Mds:                    Not affected
Vulnerability Meltdown:               Not affected
Vulnerability Mmio stale data:        Not affected
Vulnerability Reg file data sampling: Not affected
Vulnerability Retbleed:               Not affected
Vulnerability Spec rstack overflow:   Mitigation; safe RET
Vulnerability Spec store bypass:      Mitigation; Speculative Store Bypass disabled via prctl and seccomp
Vulnerability Spectre v1:             Mitigation; usercopy/swapgs barriers and __user pointer sanitization
Vulnerability Spectre v2:             Mitigation; Retpolines; IBPB conditional; IBRS_FW; STIBP always-on; RSB filling; PBRSB-eIBRS Not affected; BHI Not affected
Vulnerability Srbds:                  Not affected
Vulnerability Tsx async abort:        Not affected
__STAT__
cpu 1.22625
cpu0 0.787498
cpu1 1.07562
cpu2 1.19084
cpu3 1.2605
cpu4 1.27049
cpu5 1.3499
cpu6 1.36318
cpu7 1.35329
cpu8 1.38877
cpu9 1.34264
cpu10 1.34581
cpu11 1.33873
cpu12 1.35377
cpu13 1.25742
cpu14 1.43888
cpu15 1.29415
cpu16 1.73656
cpu17 1.33356
cpu18 1.21716
cpu19 1.14831
cpu20 1.13044
cpu21 1.2318
cpu22 1.2043
cpu23 1.16371
cpu24 1.10797
cpu25 1.07687
cpu26 1.09232
cpu27 1.0717
cpu28 1.05076
cpu29 1.10822
cpu30 0.933423
cpu31 1.22176
__PSCPU__
__END_cpu__
"#;

        let result = parse_cpu(input)?;

        if let Some(job_result) = result {
            let cpu_info: CpuInfo = job_result.value.downcast_ref::<CpuInfo>().unwrap().clone();

            assert_eq!(cpu_info.model_name, "AMD Ryzen 9 5950X 16-Core Processor");

            assert_eq!(cpu_info.core_count, 32);

            assert_eq!(cpu_info.usage_percent, 1.2262607);

            assert_eq!(cpu_info.per_core.len(), 32); // There should be 32 cores
        } else {
            panic!("Failed to parse CPU information");
        }

        Ok(())
    }
}
