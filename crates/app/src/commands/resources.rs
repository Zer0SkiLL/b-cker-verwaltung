//! Prozessor, Arbeitsspeicher und Laufzeiten für den Problembericht.
//! „Programm“ ist der eigene Prozess mit allen Nachkommen (unter Windows die WebView2-Prozesse).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use chrono::{Local, TimeZone};
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, MINIMUM_CPU_UPDATE_INTERVAL};

/// Abstand der Messungen für den Höchstwert.
const PEAK_INTERVAL: Duration = Duration::from_secs(60);

static PEAK: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesDto {
    /// Prozent 0–100.
    pub cpu_total: Option<f32>,
    pub cpu_program: Option<f32>,
    /// Bytes.
    pub mem_used: Option<u64>,
    pub mem_total: Option<u64>,
    pub mem_program: Option<u64>,
    pub mem_program_peak: Option<u64>,
    /// RFC 3339.
    pub program_since: Option<String>,
    pub computer_since: Option<String>,
}

/// Summe von Speicher und CPU über `own` und alle Nachkommen. Einträge: (pid, parent, Bytes, CPU).
pub fn tree_sum(own: u32, procs: &[(u32, Option<u32>, u64, f32)]) -> (u64, f32) {
    let mut members = vec![own];
    loop {
        let before = members.len();
        for (pid, parent, _, _) in procs {
            if parent.is_some_and(|p| members.contains(&p)) && !members.contains(pid) {
                members.push(*pid);
            }
        }
        if members.len() == before {
            break;
        }
    }
    procs
        .iter()
        .filter(|(pid, ..)| members.contains(pid))
        .fold((0, 0.0), |(mem, cpu), (_, _, m, c)| (mem + m, cpu + c))
}

pub fn record_peak(bytes: u64) {
    PEAK.fetch_max(bytes, Ordering::Relaxed);
}

pub fn peak() -> u64 {
    PEAK.load(Ordering::Relaxed)
}

fn rfc3339(unix_seconds: u64) -> Option<String> {
    Local
        .timestamp_opt(i64::try_from(unix_seconds).ok()?, 0)
        .single()
        .map(|d| d.to_rfc3339())
}

fn process_table(sys: &System) -> Vec<(u32, Option<u32>, u64, f32)> {
    sys.processes()
        .iter()
        .map(|(pid, p)| {
            (
                pid.as_u32(),
                p.parent().map(|x| x.as_u32()),
                p.memory(),
                p.cpu_usage(),
            )
        })
        .collect()
}

fn refresh_processes(sys: &mut System) {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_memory().with_cpu(),
    );
}

/// Speicher des Programm-Baums, ohne CPU-Messung (für den Hintergrund-Thread).
fn program_memory() -> Option<u64> {
    let own = sysinfo::get_current_pid().ok()?.as_u32();
    let mut sys = System::new().ok()?;
    refresh_processes(&mut sys);
    Some(tree_sum(own, &process_table(&sys)).0)
}

/// Misst alles; fehlende Werte bleiben `None`. Dauert etwa 200 ms (CPU braucht zwei Messungen).
pub fn measure() -> ResourcesDto {
    let Ok(mut sys) = System::new() else {
        return ResourcesDto {
            mem_program_peak: Some(peak()).filter(|p| *p > 0),
            ..ResourcesDto::default()
        };
    };
    let own = sysinfo::get_current_pid().ok().map(|p| p.as_u32());
    sys.refresh_memory();
    sys.refresh_cpu_usage();
    refresh_processes(&mut sys);
    std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    refresh_processes(&mut sys);
    let cores = sys.cpus().len().max(1) as f32;
    let program = own.map(|pid| tree_sum(pid, &process_table(&sys)));
    if let Some((mem, _)) = program {
        record_peak(mem);
    }
    let program_since = own
        .and_then(|pid| sys.process(sysinfo::Pid::from_u32(pid)))
        .and_then(|p| rfc3339(p.start_time()));
    ResourcesDto {
        cpu_total: Some(sys.global_cpu_usage()),
        cpu_program: program.map(|(_, cpu)| (cpu / cores).min(100.0)),
        mem_used: Some(sys.used_memory()),
        mem_total: Some(sys.total_memory()).filter(|t| *t > 0),
        mem_program: program.map(|(mem, _)| mem),
        mem_program_peak: Some(peak()).filter(|p| *p > 0),
        program_since,
        computer_since: System::boot_time().ok().and_then(rfc3339),
    }
}

/// Misst einmal pro Minute den Speicher des Programms, damit der Höchstwert stimmt.
pub fn start_peak_sampler() {
    let spawned = std::thread::Builder::new()
        .name("speicher-hoechstwert".into())
        .spawn(|| loop {
            if let Some(mem) = program_memory() {
                record_peak(mem);
            }
            std::thread::sleep(PEAK_INTERVAL);
        });
    if let Err(e) = spawned {
        log::info!("Messung des Arbeitsspeichers nicht gestartet: {e}");
    }
}
