//! Windows RAM / disk / CPU load + package temp for host gauges.
//! Samples on a background thread so the UI never waits on PowerShell.

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::win_cmd;

#[derive(Clone, Copy, Default)]
struct Sample {
    ram: Option<f64>,
    disk: Option<f64>,
    cpu: Option<f64>,
    cpu_c: Option<f64>,
}

static SAMPLE: Mutex<Sample> = Mutex::new(Sample {
    ram: None,
    disk: None,
    cpu: None,
    cpu_c: None,
});
static STARTED: AtomicBool = AtomicBool::new(false);

pub fn ram_used() -> Option<f64> {
    ensure_bg();
    SAMPLE.lock().ok().and_then(|g| g.ram)
}

pub fn disk_used() -> Option<f64> {
    ensure_bg();
    SAMPLE.lock().ok().and_then(|g| g.disk)
}

pub fn cpu_load() -> Option<f64> {
    ensure_bg();
    SAMPLE.lock().ok().and_then(|g| g.cpu)
}

/// Live CPU package °C when a sane sensor exists. Never invents a value.
/// Rejects bogus ACPI zones (e.g. PCT0 reporting ~17 °C on desktops).
pub fn cpu_temp_c() -> Option<f64> {
    ensure_bg();
    SAMPLE.lock().ok().and_then(|g| g.cpu_c)
}

fn ensure_bg() {
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    thread::Builder::new()
        .name("cf-win-gauges".into())
        .spawn(|| loop {
            let next = sample_once();
            if let Ok(mut g) = SAMPLE.lock() {
                *g = next;
            }
            thread::sleep(Duration::from_secs(3));
        })
        .ok();
}

fn sample_once() -> Sample {
    // One PowerShell process — never on the UI thread.
    // ACPI PCT0 on many boards is ambient/junk (~17 °C); only keep values in 28–105 °C.
    let text = ps(
        "$os = Get-CimInstance Win32_OperatingSystem; \
         $ram = if ($os.TotalVisibleMemorySize -gt 0) { \
           [math]::Round(1 - ($os.FreePhysicalMemory / $os.TotalVisibleMemorySize), 4) \
         } else { '' }; \
         $d = Get-PSDrive -Name C -ErrorAction SilentlyContinue; \
         $disk = if ($d -and ($d.Used + $d.Free) -gt 0) { \
           [math]::Round($d.Used / ($d.Used + $d.Free), 4) \
         } else { '' }; \
         $cpu = (Get-CimInstance Win32_Processor | Measure-Object LoadPercentage -Average).Average; \
         if ($null -eq $cpu) { $cpu = '' } else { $cpu = [math]::Round($cpu / 100, 4) }; \
         $cands = @(); \
         try { \
           Get-CimInstance -Namespace root/wmi MSAcpi_ThermalZoneTemperature -ErrorAction SilentlyContinue | \
             ForEach-Object { \
               $c = ($_.CurrentTemperature / 10.0) - 273.15; \
               if ($c -ge 28 -and $c -le 105) { $cands += $c } \
             } \
         } catch {} \
         try { \
           Get-CimInstance Win32_PerfFormattedData_Counters_ThermalZoneInformation -ErrorAction SilentlyContinue | \
             ForEach-Object { \
               if ($_.HighPrecisionTemperature -gt 2732) { \
                 $c = ($_.HighPrecisionTemperature / 10.0) - 273.15; \
                 if ($c -ge 28 -and $c -le 105) { $cands += $c } \
               } \
             } \
         } catch {} \
         $tc = ''; \
         if ($cands.Count -gt 0) { $tc = [math]::Round(($cands | Measure-Object -Maximum).Maximum, 1) }; \
         \"$ram|$disk|$cpu|$tc\"",
    );
    let Some(text) = text else {
        return Sample::default();
    };
    let mut parts = text.split('|');
    Sample {
        ram: parts.next().and_then(parse_unit),
        disk: parts.next().and_then(parse_unit),
        cpu: parts.next().and_then(parse_unit),
        cpu_c: parts.next().and_then(parse_temp_c),
    }
}

fn parse_unit(raw: &str) -> Option<f64> {
    let v: f64 = raw.trim().parse().ok()?;
    Some(v.clamp(0.0, 1.0))
}

fn parse_temp_c(raw: &str) -> Option<f64> {
    let v: f64 = raw.trim().parse().ok()?;
    // Desktop CPUs are never ~17 °C from ACPI PCT0 junk; require a sane band.
    if !(28.0..=105.0).contains(&v) {
        return None;
    }
    Some(v)
}

fn ps(script: &str) -> Option<String> {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    let out = win_cmd::hidden_capture(cmd).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_unit_clamps() {
        assert_eq!(super::parse_unit("0.5"), Some(0.5));
        assert_eq!(super::parse_unit("2"), Some(1.0));
        assert!(super::parse_unit("").is_none());
    }

    #[test]
    fn parse_temp_rejects_acpi_junk() {
        assert_eq!(super::parse_temp_c("45.5"), Some(45.5));
        assert!(super::parse_temp_c("16.9").is_none());
        assert!(super::parse_temp_c("17").is_none());
        assert!(super::parse_temp_c("").is_none());
        assert!(super::parse_temp_c("999").is_none());
    }
}
