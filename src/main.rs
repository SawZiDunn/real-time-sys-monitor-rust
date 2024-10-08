use iced::time;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, Column, Row};
use iced::{
    executor, Alignment, Application, Command, Element, Length, Settings, Subscription, Theme,
};

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::{thread, time::Duration};
use sysinfo::{Disks, Networks, System};

mod utils;

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
    ToggleProcess(bool),
}

#[derive(Serialize, Deserialize)]
struct SystemData {
    cpu_usage: f32,
    memory_usage: (u64, u64),
    disk_usage: (u64, u64),
    network_sent: u64,
    network_received: u64,
}

// basic system info
struct SystemBaseInfo {
    system_name: String,
    kernal_version: String,
    os_version: String,
    host_name: String,
}

// info for each disk
struct DisksInfo {
    name: String,
    kind: String,
    mount: String,
    total_disk: u64,
    free_disk: u64,
    used_disk_percent: f64,
}

// info for each process
struct Process {
    id: u32,
    name: String,
    cpu_usage_percent: f64,
    memory_usage_percent: f64,
}

struct SystemMonitor {
    system: System,
    disks: Disks,
    networks: Networks,
    system_base_info: SystemBaseInfo,
    // cpu info
    cpu_usage: f32,
    no_of_processes: u32,
    physical_cores: u32,
    logical_processors: u32,
    processors_info: Vec<(String, f32)>,

    // memory
    memory_usage: (u64, u64),
    swap_memory_usage: (u64, u64),

    // disk
    disk_usage: (u64, u64),
    disks_info: Vec<DisksInfo>,

    // network
    network_sent: u64,
    network_received: u64,

    // processes
    processes: Vec<Process>,

    // other
    is_monitoring: bool,
    show_cpu: bool,
    show_memory: bool,
    show_disk: bool,
    show_network: bool,
    show_process: bool,
}

impl SystemMonitor {
    fn view_metrics(&self) -> Row<Message> {
        let mut display = row![];

        // Display CPU Info
        if self.show_cpu {
            display = display.push(
                container(self.cpu_view())
                    .width(Length::FillPortion(1))
                    .padding(10),
            );
        }

        // Display Memory info
        if self.show_memory {
            let memory_view = column![
                text("Memory Usage").size(22),
                text(format!(
                    "{:.2} GB / {:.2} GB ({:.2}%)",
                    utils::convert_from_bytes(self.memory_usage.0, 3),
                    utils::convert_from_bytes(self.memory_usage.1, 3),
                    (utils::convert_from_bytes(self.memory_usage.0, 3)
                        / utils::convert_from_bytes(self.memory_usage.1, 3))
                        * 100.
                ))
                .size(20)
            ];
            display = display.push(
                container(memory_view)
                    .width(Length::FillPortion(1))
                    .padding(10),
            );
        }

        // Display Disk Usage
        if self.show_disk {
            let disk_view = column![
                text("Disk Usage").size(22),
                text(format!(
                    "{:.2} GB / {:.2} GB ({:.2}%)",
                    utils::convert_from_bytes(self.disk_usage.0, 3),
                    utils::convert_from_bytes(self.disk_usage.1, 3),
                    (utils::convert_from_bytes(self.disk_usage.0, 3)
                        / utils::convert_from_bytes(self.disk_usage.1, 3))
                        * 100.
                ))
                .size(20)
            ];
            display = display.push(
                container(disk_view)
                    .width(Length::FillPortion(1))
                    .padding(10),
            );
        }

        // Display Network Usage
        if self.show_network {
            let network_view = column![
                text("Network Usage").size(22),
                text(format!(
                    "{:.2} KB sent / {:.2} KB received",
                    utils::convert_from_bytes(self.network_sent, 1),
                    utils::convert_from_bytes(self.network_received, 1)
                ))
                .size(20)
            ];
            display = display.push(
                container(network_view)
                    .width(Length::FillPortion(1))
                    .padding(10),
            );
        }

        // Display Processes
        if self.show_process {
            let mut process_display = column![];

            for each in self.processes.iter() {
                process_display = process_display.push(text(format!(
                    "ID: {}, Name: {}, CPU: {:.2}%, Memory: {:.2}%",
                    each.id, each.name, each.cpu_usage_percent, each.memory_usage_percent
                )));
            }

            let scrollable_processes = scrollable(process_display).height(Length::FillPortion(3));
            display = display.push(
                container(scrollable_processes)
                    .width(Length::FillPortion(1))
                    .padding(10),
            );
        }

        display.spacing(20)
    }

    fn view_sys_base_info(&self) -> Column<Message> {
        // System info line
        let system_base_info = text(format!(
            "System Name: {} - OS Version: {} - Kernel Version: {} - Host Name: {}",
            self.system_base_info.system_name,
            self.system_base_info.os_version,
            self.system_base_info.kernal_version,
            self.system_base_info.host_name
        ))
        .size(22)
        .style(iced::theme::Text::Color(iced::Color::BLACK));

        // Align the text centrally, with padding for aesthetics
        column!(system_base_info)
            .spacing(10)
            .padding(10)
            .width(Length::Fill)
            .align_items(Alignment::Center)
    }

    fn cpu_view(&self) -> Element<Message> {
        // Header for CPU Usage
        let header = text("CPU Usage")
            .size(22)
            .style(iced::theme::Text::Color(iced::Color::BLACK));

        // Main CPU information (total CPU, processes, cores)
        let cpu_info = text(format!(
            "Total CPU Usage: {:.2}%\nProcesses: {}\nPhysical Cores: {}\nLogical Cores: {}",
            self.cpu_usage, self.no_of_processes, self.physical_cores, self.logical_processors
        ))
        .size(20)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(
            0.8, 0.1, 0.1,
        )));

        // Display usage for each individual core
        let per_core_usage: Column<_> = self
            .processors_info
            .iter()
            .fold(Column::new(), |col, (name, usage)| {
                col.push(text(format!("{}: {:.2}%", name, usage)).size(18))
            });

        // Create a container for the CPU view
        column![header, cpu_info, per_core_usage]
            .spacing(10)
            .padding(10)
            .width(Length::FillPortion(1))
            .into()
    }
}

impl Application for SystemMonitor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut system = System::new_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        system.refresh_all();

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let total_swap_memory = system.total_swap();
        let used_swap_memory = system.used_swap();

        let total_disk = disks.iter().fold(0, |acc, disk| acc + disk.total_space());
        let used_disk = disks.iter().fold(0, |acc, disk| {
            acc + (disk.total_space() - disk.available_space())
        });

        let disks_info: Vec<DisksInfo> = disks
            .list()
            .iter()
            .map(|disk| DisksInfo {
                name: String::from(disk.name().to_string_lossy()),
                kind: disk.kind().to_string(),
                mount: disk.mount_point().to_string_lossy().to_string(),
                total_disk: disk.total_space(),
                free_disk: disk.available_space(),
                used_disk_percent: if total_disk > 0 {
                    ((disk.total_space() as f64 - disk.available_space() as f64)
                        / disk.total_space() as f64)
                        * 100.
                } else {
                    0.0
                },
            })
            .collect();

        let network_sent = networks.iter().fold(0, |acc, (_interface, network)| {
            acc + network.total_transmitted()
        });

        let network_received = networks.iter().fold(0, |acc, (_interface, network)| {
            acc + network.total_received()
        });

        let no_of_processes: u32 = system.processes().len() as u32;
        let cpu_usage = system.global_cpu_usage();
        let physical_cores: u32 = match system.physical_core_count() {
            Some(count) => count as u32,
            None => {
                eprintln!("Failed to retrieve physical core count. Using default value of 0.");
                0
            }
        };
        let logical_processors = system.cpus().len() as u32;

        let system_base_info = SystemBaseInfo {
            system_name: System::name().unwrap_or_default(),
            kernal_version: System::kernel_version().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            host_name: System::host_name().unwrap_or_default(),
        };

        let processors_info = system
            .cpus()
            .iter()
            .map(|cpu| (cpu.name().to_string(), cpu.cpu_usage()))
            .collect();

        // Capture initial data to calculate CPU percentages accurately
        let initial_cpu_time = system.global_cpu_usage();

        // Optional delay for measuring changes over time
        thread::sleep(Duration::from_secs(1));

        // Refresh system data after sleep for updated values
        system.refresh_all();

        // Calculate the change in CPU usage over time
        let cpu_time_diff = system.global_cpu_usage() - initial_cpu_time;

        let mut processes: Vec<Process> = Vec::new();

        // Iterate through each process and display relevant resource metrics
        for (pid, process) in system.processes() {
            // Calculate CPU usage percent for the process
            let cpu_usage_percent = if cpu_time_diff > 0.0 {
                (process.cpu_usage() / cpu_time_diff) * 100.0
            } else {
                0.0
            };

            // Calculate memory usage percent relative to total system memory
            let memory_usage_percent =
                (process.memory() as f64 / system.total_memory() as f64) * 100.0;
            processes.push(Process {
                id: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage_percent: cpu_usage_percent as f64,
                memory_usage_percent: memory_usage_percent,
            });
        }

        (
            SystemMonitor {
                system,
                disks,
                disks_info,
                networks,
                system_base_info,
                cpu_usage,
                no_of_processes,
                processors_info,
                physical_cores,
                logical_processors,
                memory_usage: (used_memory, total_memory),
                swap_memory_usage: (used_swap_memory, total_swap_memory),
                disk_usage: (used_disk, total_disk),
                network_sent,
                network_received,
                processes,
                is_monitoring: false,
                show_cpu: true,
                show_memory: true,
                show_disk: true,
                show_network: true,
                show_process: true,
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

                    self.cpu_usage = self.system.global_cpu_usage();
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
                    self.swap_memory_usage = (self.system.used_swap(), self.system.total_swap());

                    self.disk_usage = (
                        self.disks.iter().fold(0, |acc, disk| {
                            acc + (disk.total_space() - disk.available_space())
                        }),
                        self.disks
                            .iter()
                            .fold(0, |acc, disk| acc + disk.total_space()),
                    );

                    self.disks_info = self
                        .disks
                        .iter()
                        .map(|disk| DisksInfo {
                            name: String::from(disk.name().to_string_lossy()),
                            kind: disk.kind().to_string(),
                            mount: disk.mount_point().to_string_lossy().to_string(),
                            total_disk: disk.total_space(),
                            free_disk: disk.available_space(),
                            used_disk_percent: ((disk.total_space() as f64
                                - disk.available_space() as f64)
                                / disk.total_space() as f64)
                                * 100.,
                        })
                        .collect();

                    self.network_sent =
                        self.networks.iter().fold(0, |acc, (_interface, network)| {
                            acc + network.total_transmitted()
                        });
                    self.network_received =
                        self.networks.iter().fold(0, |acc, (_interface, network)| {
                            acc + network.total_received()
                        });

                    self.processes.clear();
                    for (pid, process) in self.system.processes() {
                        // Calculate memory usage percent relative to total system memory
                        let memory_usage_percent =
                            (process.memory() as f64 / self.system.total_memory() as f64) * 100.0;

                        self.processes.push(Process {
                            id: pid.as_u32(),
                            name: process.name().to_string_lossy().to_string(),
                            cpu_usage_percent: process.cpu_usage() as f64,
                            memory_usage_percent,
                        });
                    }

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
            Message::ToggleProcess(value) => self.show_process = value,
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        // Monitoring button on the left, styled similar to the image
        let monitoring_button = button(
            text(if self.is_monitoring {
                "Stop Monitoring"
            } else {
                "Start Monitoring"
            })
            .size(20),
        )
        .padding(10)
        .width(Length::Fixed(150.))
        .on_press(Message::ToggleMonitoring);

        // Customizable display checkboxes (aligned vertically on the left)
        let cpu_checkbox = checkbox("CPU Usage", self.show_cpu).on_toggle(Message::ToggleCpu);
        let memory_checkbox =
            checkbox("Memory Usage", self.show_memory).on_toggle(Message::ToggleMemory);
        let disk_checkbox = checkbox("Disk Usage", self.show_disk).on_toggle(Message::ToggleDisk);
        let network_checkbox =
            checkbox("Network Usage", self.show_network).on_toggle(Message::ToggleNetwork);
        let process_checkbox =
            checkbox("Processes", self.show_process).on_toggle(Message::ToggleProcess);

        let col_left = column![
            monitoring_button,
            column![
                cpu_checkbox,
                memory_checkbox,
                disk_checkbox,
                network_checkbox,
                process_checkbox
            ]
            .spacing(10)
        ]
        .spacing(20)
        .align_items(Alignment::Start)
        .padding(20);

        // System Information (spanning across the top center, as seen in the image)
        let sys_info_row = self
            .view_sys_base_info()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .padding(10);

        // The main metrics area (center, for metrics display)
        let metrics_row = self
            .view_metrics()
            .width(Length::Fill)
            .spacing(20)
            .padding(10)
            .align_items(Alignment::Center);

        // Combine the layout (left panel and main display area)
        let content = row![
            col_left.width(Length::Fixed(200.)),
            column![sys_info_row, metrics_row].spacing(10)
        ];

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
