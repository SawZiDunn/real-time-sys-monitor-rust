use iced::time;
use iced::widget::{button, checkbox, column, container, row, text, Column};
use iced::{
    executor, Alignment, Application, Command, Element, Length, Settings, Subscription, Theme,
};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, System, SystemExt};

fn main() -> iced::Result {
    SystemMonitor::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ToggleMonitoring,
    ToggleCpu(bool),
    ToggleMemory(bool),
    ToggleDisk(bool),
    ToggleNetwork(bool),
}

#[derive(Serialize, Deserialize)]
struct SystemData {
    cpu_usage: f32,
    memory_usage: (u64, u64),
    disk_usage: (u64, u64),
    network_sent: u64,
    network_received: u64,
}

struct SystemBaseInfo {
    system_name: String,
    kernal_version: String,
    os_version: String,
    host_name: String,
}

struct SystemMonitor {
    system: System,
    system_base_info: SystemBaseInfo,
    // cpu info
    cpu_usage: f32,
    no_of_processes: u32,
    physical_cores: u32,
    logical_processors: u32,
    processors_info: Vec<(String, f32)>,

    // memory
    memory_usage: (u64, u64),
    disk_usage: (u64, u64),

    network_sent: u64,
    network_received: u64,

    is_monitoring: bool,
    show_cpu: bool,
    show_memory: bool,
    show_disk: bool,
    show_network: bool,
}

impl SystemMonitor {
    fn view_metrics(&self) -> Column<Message> {
        let mut display = column![];

        let system_base_info = text(format!(
            "System Name: {}, OS Version: {}, Kernal Version: {}, Host Name: {}",
            self.system_base_info.system_name,
            self.system_base_info.os_version,
            self.system_base_info.kernal_version,
            self.system_base_info.host_name
        ))
        .size(20);

        display = display.push(system_base_info);

        // Display CPU usage and details for each processor
        if self.show_cpu {
            let cpu_info = text(format!(
                "CPU usage: {:.2}%\n\
                 Number of processes: {}\n\
                 Physical Cores: {}\n\
                 Logical processors: {}",
                self.cpu_usage, self.no_of_processes, self.physical_cores, self.logical_processors
            ))
            .size(20);

            display = display.push(cpu_info);

            // Loop through `processors_info` and add each processor's details
            for (name, usage) in self.processors_info.iter() {
                let processor_text = text(format!("{}: - Usage: {:.2}%", name, usage)).size(20);
                display = display.push(processor_text);
            }
        }

        // Display Memory Usage
        if self.show_memory {
            let used_gb = self.memory_usage.0 as f64 / f64::powf(1024., 3.);
            let total_gb = self.memory_usage.1 as f64 / f64::powf(1024., 3.);
            let used_percent = (self.memory_usage.0 as f64 / self.memory_usage.1 as f64) * 100.;

            let memory_info = text(format!(
                "Memory usage: {:.2} GB / {:.2} GB ({:.2})%",
                used_gb, total_gb, used_percent
            ))
            .size(20);
            display = display.push(memory_info);
        }

        // Display Disk Usage
        if self.show_disk {
            let used_gb = self.disk_usage.0 as f64 / f64::powf(1024., 3.);
            let total_gb = self.disk_usage.1 as f64 / f64::powf(1024., 3.);
            let used_percent = (self.disk_usage.0 as f64 / self.disk_usage.1 as f64) * 100.;

            let disk_info = text(format!(
                "Disk usage: {:.2} GB / {:.2} GB ({:.2})%",
                used_gb, total_gb, used_percent
            ))
            .size(20);
            display = display.push(disk_info);
        }

        // Display Network Usage
        if self.show_network {
            let network_info = text(format!(
                "Network usage: {:.2} MB sent / {:.2} MB received",
                self.network_sent as f64 / u64::pow(10, 6) as f64,
                self.network_received as f64 / u64::pow(10, 6) as f64
            ))
            .size(20);
            display = display.push(network_info);
        }

        display.spacing(20)
    }
}

impl Application for SystemMonitor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut system = System::new_all();
        system.refresh_all();

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let total_disk = system
            .disks()
            .iter()
            .fold(0, |acc, disk| acc + disk.total_space());
        let used_disk = system
            .disks()
            .iter()
            .fold(0, |acc, disk| acc + disk.available_space());

        let network_sent = system
            .networks()
            .iter()
            .fold(0, |acc, (_interface, data)| acc + data.transmitted());
        let network_received = system
            .networks()
            .iter()
            .fold(0, |acc, (_interface, data)| acc + data.received());

        let no_of_processes: u32 = system.processes().len() as u32;
        let cpu_usage = system.global_cpu_info().cpu_usage();
        let physical_cores: u32 = system.physical_core_count().unwrap_or(0) as u32;
        let logical_processors = system.cpus().len() as u32;

        let system_base_info = SystemBaseInfo {
            system_name: system.name().unwrap_or_default(),
            kernal_version: system.kernel_version().unwrap_or_default(),
            os_version: system.os_version().unwrap_or_default(),
            host_name: system.host_name().unwrap_or_default(),
        };

        let processors_info = system
            .cpus()
            .iter()
            .map(|cpu| (cpu.name().to_string(), cpu.cpu_usage()))
            .collect();

        (
            SystemMonitor {
                system,
                system_base_info,
                cpu_usage,
                no_of_processes,
                processors_info,
                physical_cores,
                logical_processors,
                memory_usage: (used_memory, total_memory),
                disk_usage: (used_disk, total_disk),
                network_sent,
                network_received,
                is_monitoring: false,
                show_cpu: true,
                show_memory: true,
                show_disk: true,
                show_network: true,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("System Monitor")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => {
                if self.is_monitoring {
                    self.system.refresh_all();

                    self.cpu_usage = self.system.global_cpu_info().cpu_usage();
                    self.no_of_processes = self.system.processes().len() as u32;
                    self.processors_info = self
                        .system
                        .cpus()
                        .iter()
                        .map(|cpu| (cpu.name().to_string(), cpu.cpu_usage()))
                        .collect();

                    self.physical_cores = self.system.physical_core_count().unwrap_or(0) as u32;
                    self.logical_processors = self.system.cpus().len() as u32;

                    self.memory_usage = (self.system.used_memory(), self.system.total_memory());
                    self.disk_usage = (
                        self.system.disks().iter().fold(0, |acc, disk| {
                            acc + (disk.total_space() - disk.available_space())
                        }),
                        self.system
                            .disks()
                            .iter()
                            .fold(0, |acc, disk| acc + disk.total_space()),
                    );

                    self.network_sent = self
                        .system
                        .networks()
                        .iter()
                        .fold(0, |acc, (_interface, data)| acc + data.transmitted());
                    self.network_received = self
                        .system
                        .networks()
                        .iter()
                        .fold(0, |acc, (_interface, data)| acc + data.received());

                    log_metrics(&self);
                }
            }
            Message::ToggleMonitoring => {
                self.is_monitoring = !self.is_monitoring;
            }
            Message::ToggleCpu(value) => self.show_cpu = value,
            Message::ToggleMemory(value) => self.show_memory = value,
            Message::ToggleDisk(value) => self.show_disk = value,
            Message::ToggleNetwork(value) => self.show_network = value,
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        // Monitoring button
        let monitoring_button = button(
            text(if self.is_monitoring {
                "Stop Monitoring"
            } else {
                "Start Monitoring"
            })
            .size(20),
        )
        .on_press(Message::ToggleMonitoring);

        // Customizable display checkboxes
        let cpu_checkbox = checkbox("Show CPU Usage", self.show_cpu).on_toggle(Message::ToggleCpu);
        let memory_checkbox =
            checkbox("Show Memory Usage", self.show_memory).on_toggle(Message::ToggleMemory);
        let disk_checkbox =
            checkbox("Show Disk Usage", self.show_disk).on_toggle(Message::ToggleDisk);
        let network_checkbox =
            checkbox("Show Network Usage", self.show_network).on_toggle(Message::ToggleNetwork);

        let content = column![
            monitoring_button,
            row![cpu_checkbox, memory_checkbox].spacing(20),
            row![disk_checkbox, network_checkbox].spacing(20),
            self.view_metrics()
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.is_monitoring {
            time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

fn log_metrics(system_monitor: &SystemMonitor) {
    let data = SystemData {
        cpu_usage: system_monitor.cpu_usage,
        memory_usage: system_monitor.memory_usage,
        disk_usage: system_monitor.disk_usage,
        network_sent: system_monitor.network_sent,
        network_received: system_monitor.network_received,
    };

    let serialized = serde_json::to_string(&data).expect("Failed to serialize system data");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("system_log.json")
        .expect("Failed to open log file");

    writeln!(file, "{}", serialized).expect("Failed to write data to log file");
}
