use iced::time;
use iced::widget::{button, checkbox, Column, Container, Row, Text};
use iced::{executor, Alignment, Application, Command, Element, Length, Settings, Subscription};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, System, SystemExt};
use tokio::time::interval;

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

// for logging data
#[derive(Serialize, Deserialize)]
struct SystemData {
    cpu_usage: f32,
    memory_usage: (u64, u64),
    disk_usage: (u64, u64),
    network_sent: u64,
    network_received: u64,
}

struct SystemMonitor {
    system: System,
    // cpu
    cpu_usage: f32,
    no_of_processes: u32,
    base_cpu_speed: u64,
    physical_cores: u32,
    logical_processors: u32,
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

impl Application for SystemMonitor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
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

        // Summing up the network data
        let network_sent = system
            .networks()
            .iter()
            .fold(0, |acc, (_interface, data)| acc + data.transmitted());
        let network_received = system
            .networks()
            .iter()
            .fold(0, |acc, (_interface, data)| acc + data.received());

        let no_of_processes: u32 = system.processes().len() as u32;
        let base_cpu_speed: u64 = system.global_cpu_info().frequency();
        let cpu_usage = system.global_cpu_info().cpu_usage();
        let physical_cores: u32 = system.physical_core_count().unwrap_or(0) as u32;
        let logical_processors = system.cpus().len() as u32;

        (
            SystemMonitor {
                system,
                cpu_usage,
                no_of_processes,
                base_cpu_speed,
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
                    self.base_cpu_speed = self.system.global_cpu_info().frequency();

                    self.physical_cores = self.system.physical_core_count().unwrap_or(0) as u32;
                    self.logical_processors = self.system.cpus().len() as u32;

                    self.memory_usage = (self.system.used_memory(), self.system.total_memory());
                    self.disk_usage = (
                        self.system
                            .disks()
                            .iter()
                            .fold(0, |acc, disk| acc + disk.available_space()),
                        self.system
                            .disks()
                            .iter()
                            .fold(0, |acc, disk| acc + disk.total_space()),
                    );

                    // Updating network data
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

                    // Log metrics to a file
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
            Text::new(if self.is_monitoring {
                "Stop Monitoring"
            } else {
                "Start Monitoring"
            })
            .size(20),
        )
        .on_press(Message::ToggleMonitoring);

        // Customizable display checkboxes
        let cpu_checkbox = checkbox("Show CPU Usage", self.show_cpu, Message::ToggleCpu).size(20);
        let memory_checkbox =
            checkbox("Show Memory Usage", self.show_memory, Message::ToggleMemory).size(20);
        let disk_checkbox =
            checkbox("Show Disk Usage", self.show_disk, Message::ToggleDisk).size(20);
        let network_checkbox = checkbox(
            "Show Network Usage",
            self.show_network,
            Message::ToggleNetwork,
        )
        .size(20);

        // Metrics display
        let mut display = Column::new().spacing(20);

        // Metrics display
        if self.show_cpu {
            let cpu_info = Text::new(format!(
                "CPU usage: {:.2}%\n
                Number of proceses: {}\n
                CPU Base Speed: {} MHz\n
                Physical Cores: {}\n
                Logical processors: {}",
                self.cpu_usage,
                self.no_of_processes,
                self.base_cpu_speed,
                self.physical_cores,
                self.logical_processors
            ))
            .size(30);
            display = display.push(cpu_info);
        }
        if self.show_memory {
            let memory_info = Text::new(format!(
                "Memory usage: {} KB / {} KB",
                self.memory_usage.0, self.memory_usage.1
            ))
            .size(30);
            display = display.push(memory_info);
        }
        if self.show_disk {
            let disk_info = Text::new(format!(
                "Disk usage: {} KB / {} KB",
                self.disk_usage.0, self.disk_usage.1
            ))
            .size(30);
            display = display.push(disk_info);
        }
        if self.show_network {
            let network_info = Text::new(format!(
                "Network usage: {} bytes sent / {} bytes received",
                self.network_sent, self.network_received
            ))
            .size(30);
            display = display.push(network_info);
        }

        // Create rows for the layout
        let checkboxes_row = Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(cpu_checkbox)
            .push(memory_checkbox);

        let metrics_row = Row::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(disk_checkbox)
            .push(network_checkbox);

        // Layout: Start/Stop button -> Rows of Checkboxes -> Metrics
        let content = Column::new()
            .spacing(20)
            .align_items(Alignment::Center)
            .push(monitoring_button)
            .push(checkboxes_row) // First row of checkboxes
            .push(metrics_row) // Second row of checkboxes
            .push(display); // Metrics display

        Container::new(content)
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

// Function to log system metrics to a file
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
