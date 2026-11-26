use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessSort {
    Cpu,
    Memory,
    Pid,
}

impl ProcessSort {
    pub fn next(self) -> Self {
        match self {
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Pid,
            Self::Pid => Self::Cpu,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU%",
            Self::Memory => "MEM",
            Self::Pid => "PID",
        }
    }
}

pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub virtual_memory: u64,
    pub status: String,
}

pub struct ProcessListInfo {
    pub total_count: usize,
    pub processes: Vec<ProcessInfo>,
}

pub struct ProcessCollector {
    sys: System,
}

impl ProcessCollector {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .without_tasks(),
        );

        Self { sys }
    }

    pub fn collect(&mut self, sort: ProcessSort) -> ProcessListInfo {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .without_tasks(),
        );

        let mut processes: Vec<ProcessInfo> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, proc_data)| ProcessInfo {
                pid: pid.as_u32(),
                name: proc_data.name().to_string_lossy().into_owned(),
                cpu_usage: proc_data.cpu_usage().max(0.0),
                memory: proc_data.memory(),
                virtual_memory: proc_data.virtual_memory(),
                status: format!("{:?}", proc_data.status()),
            })
            .collect();

        sort_processes(&mut processes, sort);

        ProcessListInfo {
            total_count: processes.len(),
            processes,
        }
    }
}

pub fn sort_processes(processes: &mut [ProcessInfo], sort: ProcessSort) {
    match sort {
        ProcessSort::Cpu => processes.sort_by(|a, b| {
            b.cpu_usage
                .total_cmp(&a.cpu_usage)
                .then_with(|| b.memory.cmp(&a.memory))
                .then_with(|| a.pid.cmp(&b.pid))
        }),
        ProcessSort::Memory => processes.sort_by(|a, b| {
            b.memory
                .cmp(&a.memory)
                .then_with(|| b.cpu_usage.total_cmp(&a.cpu_usage))
                .then_with(|| a.pid.cmp(&b.pid))
        }),
        ProcessSort::Pid => processes.sort_by(|a, b| a.pid.cmp(&b.pid)),
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessCollector, ProcessInfo, ProcessSort, sort_processes};

    #[test]
    fn collects_running_processes() {
        let mut collector = ProcessCollector::new();
        let list = collector.collect(ProcessSort::Cpu);
        assert!(list.total_count > 0);
        assert_eq!(list.total_count, list.processes.len());
    }

    #[test]
    fn sorts_processes_by_cpu_memory_and_pid() {
        let make_proc = |pid: u32, cpu: f32, mem: u64| ProcessInfo {
            pid,
            name: format!("proc-{}", pid),
            cpu_usage: cpu,
            memory: mem,
            virtual_memory: mem * 2,
            status: "Run".to_string(),
        };

        let mut list = vec![
            make_proc(30, 5.0, 300),
            make_proc(10, 25.0, 100),
            make_proc(20, 10.0, 500),
        ];

        sort_processes(&mut list, ProcessSort::Cpu);
        assert_eq!(list.iter().map(|p| p.pid).collect::<Vec<_>>(), vec![10, 20, 30]);

        sort_processes(&mut list, ProcessSort::Memory);
        assert_eq!(list.iter().map(|p| p.pid).collect::<Vec<_>>(), vec![20, 30, 10]);

        sort_processes(&mut list, ProcessSort::Pid);
        assert_eq!(list.iter().map(|p| p.pid).collect::<Vec<_>>(), vec![10, 20, 30]);
    }

    #[test]
    fn cycles_process_sort_modes() {
        assert_eq!(ProcessSort::Cpu.next(), ProcessSort::Memory);
        assert_eq!(ProcessSort::Memory.next(), ProcessSort::Pid);
        assert_eq!(ProcessSort::Pid.next(), ProcessSort::Cpu);
    }
}
