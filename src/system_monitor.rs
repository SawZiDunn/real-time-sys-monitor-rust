use crate::models::{DisksInfo, Message, Process, SystemBaseInfo, SystemMonitor};
use crate::utils::{calculate_disk_usage, convert_from_bytes, log_metrics};
use iced::time;
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, row, scrollable, text, Column, TextInput,
};
use iced::{executor, Alignment, Application, Command, Element, Length, Subscription, Theme};
use std::{thread, time::Duration};
use sysinfo::{Disks, Networks, System};

impl SystemMonitor {
    fn create_control_row(&self) -> Element<Message> {
        let interval_input = TextInput::new("Interval(s)", self.interval_in_secs.trim())
            .padding(10)
            .width(Length::Fixed(120.0))
            .on_input(|s| Message::IntervalChanged(s));

        let monitoring_button = button(
            text(if self.is_monitoring {
                "Stop Monitoring"
            } else {
                "Start Monitoring"
            })
            .size(14),
        )
        .padding(10)
        .width(Length::Fixed(200.))
        .on_press(Message::ToggleMonitoring);

        let save_checkbox = checkbox("Save To File", self.save_to_file)
            .spacing(8)
            .on_toggle(Message::ToggleSaveToFile);

        row![interval_input, monitoring_button, save_checkbox]
            .spacing(20)
            .align_items(Alignment::Center)
            .padding(20)
            .into()
    }

    fn view_sys_base_info(&self) -> Column<Message> {
        // System info line
        let system_base_info = text(format!(
            "System Name: {} | OS Version: {} | Kernel Version: {} | Host: {}",
            self.system_base_info.system_name,
            self.system_base_info.os_version,
            self.system_base_info.kernal_version,
            self.system_base_info.host_name
        ))
        .size(20)
        .style(iced::theme::Text::Color(iced::Color::from_rgb(
            1.0, 0.92, 0.0,
        )));

        // align the text centrally, with padding for better look
        column!(system_base_info)
            .spacing(20)
            .padding(15)
            .width(Length::Fill)
            .align_items(Alignment::Center)
    }

    fn view_cpu_info(&self) -> Column<Message> {
        column![
            text("CPU Usage\n")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.2, 0.6, 1.0,
                ))),
            text("---------------")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2,
                ))),
            text(format!("Total: {:.2}%", self.cpu_usage))
                .size(18)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2
                ))),
            text(format!("Processes: {}", self.no_of_processes))
                .size(18)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2
                ))),
            text(format!("Plysical Cores: {}", self.physical_cores))
                .size(18)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2
                ))),
            text(format!("Logical Processors: {}", self.logical_processors))
                .size(18)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2
                ))),
            text("\n"),
            horizontal_rule(5),
            text("\n"),
            self.view_per_core_usage(),
        ]
        .padding(5)
    }

    fn view_memory_info(&self) -> Column<Message> {
        column![
            text("Memory Usage\n")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.2, 0.6, 1.0,
                ))),
            text("------------\n")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2,
                ))),
            text(format!(
                "{:.2} GB / {:.2} GB ({:.2}%)",
                convert_from_bytes(self.memory_usage.0, 3),
                convert_from_bytes(self.memory_usage.1, 3),
                (convert_from_bytes(self.memory_usage.0, 3)
                    / convert_from_bytes(self.memory_usage.1, 3))
                    * 100.
            ))
            .size(16)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.1, 0.8, 0.2,
            ))),
            text("\nSwap Memory Usage\n")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.2, 0.6, 1.0,
                ))),
            text("------------\n")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2,
                ))),
            text(format!(
                "{:.2} GB / {:.2} GB ({:.2}%)",
                convert_from_bytes(self.swap_memory_usage.0, 3),
                convert_from_bytes(self.swap_memory_usage.1, 3),
                (convert_from_bytes(self.swap_memory_usage.0, 3)
                    / convert_from_bytes(self.swap_memory_usage.1, 3))
                    * 100.
            ))
            .size(16)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.1, 0.8, 0.2,
            )))
        ]
        .padding(10)
    }

    fn view_disk_info(&self) -> Column<Message> {
        let mut disk_display = column![
            text("Disk Usage")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.2, 0.6, 1.0,
                ))),
            text("------------\n")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2,
                ))),
            text(format!(
                "Total Disk Usage: {:.2} GB / {:.2} GB ({:.2}%)",
                convert_from_bytes(self.disk_usage.0, 3),
                convert_from_bytes(self.disk_usage.1, 3),
                (convert_from_bytes(self.disk_usage.0, 3)
                    / convert_from_bytes(self.disk_usage.1, 3))
                    * 100.
            ))
            .size(16)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.1, 0.8, 0.2,
            )))
        ];

        for disk in &self.disks_info {
            let disk_info = column![
                text(format!("Disk Name: {}\n", disk.name)).size(20).style(
                    iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.6, 1.0))
                ),
                text(format!("Type: {}", disk.kind))
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.1, 0.8, 0.2,
                    ))),
                text(format!("Mount Point: {}", disk.mount)).size(16).style(
                    iced::theme::Text::Color(iced::Color::from_rgb(0.1, 0.8, 0.2,))
                ),
                text(format!(
                    "Total Disk Space: {:.2} GB",
                    convert_from_bytes(disk.total_disk, 3)
                ))
                .size(18)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2,
                ))),
                text(format!(
                    "Free Disk Space: {:.2} GB",
                    convert_from_bytes(disk.free_disk, 3)
                ))
                .size(18)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2,
                ))),
                text(format!("Used Disk: {:.2}%", disk.used_disk_percent))
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.1, 0.8, 0.2,
                    ))),
                // add a little space between each disk
                text("-------------------------------------------------")
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.2, 0.6, 1.0,
                    ))),
            ];

            // Add the disk info to the main display column
            disk_display = disk_display.push(container(disk_info).padding(10));
        }

        // return display column
        disk_display
    }

    fn view_network_info(&self) -> Column<Message> {
        column![
            text("Network Usage\n")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.2, 0.6, 1.0,
                ))),
            text("---------------")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2,
                ))),
            text(format!(
                "- Sent: {:.2} KB\n\n- Received: {:.2} KB",
                convert_from_bytes(self.network_sent, 1),
                convert_from_bytes(self.network_received, 1)
            ))
            .size(16)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.1, 0.8, 0.2,
            )))
        ]
    }

    fn view_per_core_usage(&self) -> Column<Message> {
        self.processors_info
            .iter()
            .fold(Column::new(), |col, (name, usage)| {
                col.push(text(format!("{}: {:.2}%", name, usage)).size(16).style(
                    iced::theme::Text::Color(iced::Color::from_rgb(0.1, 0.8, 0.2)),
                ))
            })
    }

    fn view_process_info(&self) -> Column<Message> {
        let mut process_display = column![
            text("Running Processes")
                .size(24)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.2, 0.6, 1.0
                ))),
            text("---------------")
                .size(22)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.1, 0.8, 0.2,
                ))),
        ];
        // 0.1, 0.8, 0.2
        for each in self.processes.iter() {
            // slicing the running process name if it's too long
            let name = if each.name.len() > 40 {
                &each.name[..38]
            } else {
                &each.name
            };

            process_display = process_display.push(
                row![
                    text(format!("ID: {} |", each.id)).style(iced::theme::Text::Color(
                        iced::Color::from_rgb(0.1, 0.8, 0.2)
                    ),),
                    text(format!("Name: {} |", name)).style(iced::theme::Text::Color(
                        iced::Color::from_rgb(0.1, 0.8, 0.2)
                    ),),
                    text(format!("CPU: {:.2}% |", each.cpu_usage_percent)).style(
                        iced::theme::Text::Color(iced::Color::from_rgb(0.1, 0.8, 0.2)),
                    ),
                    text(format!("Memory: {:.2}%", each.memory_usage_percent)).style(
                        iced::theme::Text::Color(iced::Color::from_rgb(0.1, 0.8, 0.2)),
                    )
                ]
                .spacing(10),
            );
        }

        process_display
    }
}

impl Application for SystemMonitor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut system = System::new_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        system.refresh_all();

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let total_swap_memory = system.total_swap();
        let used_swap_memory = system.used_swap();

        let (used_disk, total_disk) = calculate_disk_usage(&disks);

        let disks_info: Vec<DisksInfo> = disks
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

        // capture initial data to calculate CPU percentages accurately
        let initial_cpu_time = system.global_cpu_usage();

        // Optional delay for measuring changes over time
        thread::sleep(Duration::from_secs(1));

        // Refresh system data after sleep for updated values
        system.refresh_cpu_all();
        system.refresh_memory();

        // Calculate the change in CPU usage over time
        let cpu_time_diff = system.global_cpu_usage() - initial_cpu_time;

        let mut processes: Vec<Process> = Vec::new();

        // Iterate through each process
        for (pid, process) in system.processes() {
            // Calculate CPU usage percent for the process
            let cpu_usage_percent = if cpu_time_diff > 0.0 {
                process.cpu_usage()
            } else {
                0.0
            };

            // Calculate memory usage percent relative to total system memory
            let memory_usage_percent =
                (process.memory() as f64 / system.total_memory() as f64) * 100.0;

            if memory_usage_percent >= 0.01 {
                processes.push(Process {
                    id: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cpu_usage_percent: cpu_usage_percent as f64,
                    memory_usage_percent: memory_usage_percent,
                });
            }
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
                save_to_file: false,
                interval_in_secs: "".to_string(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Real-Time System Monitor")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick => {
                if self.is_monitoring {
                    self.system.refresh_all();

                    // update cpu info
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

                    // update memory info
                    self.memory_usage = (self.system.used_memory(), self.system.total_memory());
                    self.swap_memory_usage = (self.system.used_swap(), self.system.total_swap());

                    // update disk info
                    self.disk_usage = calculate_disk_usage(&self.disks);

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

                    // update network info
                    self.network_sent =
                        self.networks.iter().fold(0, |acc, (_interface, network)| {
                            acc + network.total_transmitted()
                        });
                    self.network_received =
                        self.networks.iter().fold(0, |acc, (_interface, network)| {
                            acc + network.total_received()
                        });

                    // update processes
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

                    self.processes.sort_by(|a, b| {
                        b.memory_usage_percent
                            .partial_cmp(&a.memory_usage_percent)
                            .unwrap_or(std::cmp::Ordering::Less)
                    });
                }
            }

            Message::LogToFile => {
                log_metrics(&self);
            }

            Message::ToggleSaveToFile(x) => {
                self.save_to_file = x;
            }
            Message::ToggleMonitoring => {
                self.is_monitoring = !self.is_monitoring;
            }

            Message::IntervalChanged(x) => {
                self.interval_in_secs = x;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let control_row = self.create_control_row();

        // system information row
        let sys_info_row = self
            .view_sys_base_info()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .padding(10);

        // Create a column for each info category: CPU, Memory, Disk, Network, Processes
        let cpu_info = self.view_cpu_info().padding(5);
        let memory_info = self.view_memory_info().padding(5);
        let disk_info = self.view_disk_info().padding(10);
        let network_info = self.view_network_info().padding(5);
        let process_info = self.view_process_info().padding(5);

        let scrollable_process = scrollable(process_info).height(Length::FillPortion(3));

        // Arrange these categories in a row with proper spacing
        let metrics_row = row![
            cpu_info,
            column!(memory_info, network_info).padding(5),
            disk_info,
            scrollable_process
        ]
        .spacing(15)
        .padding(5)
        .align_items(Alignment::Center);

        // Combine the layout
        let content = column![control_row, sys_info_row, metrics_row]
            .spacing(20)
            .align_items(Alignment::Center)
            .padding(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if self.is_monitoring {
            // Parse interval string to u64
            let interval_secs = match self.interval_in_secs.parse::<u64>() {
                Ok(x) => x,
                Err(_) => {
                    // Default to 3 second if parsing fails
                    eprintln!("Invalid Value for Logging Interval!\n3 seconds interval time will be used by default.");
                    3
                }
            };

            let tick_interval = time::every(Duration::from_secs(1)).map(|_| Message::Tick);
            if self.interval_in_secs.is_empty() {
                eprintln!("Input value for logging interval is empty!\nSystem Data will not be saved to the file.")
            }

            if self.save_to_file && !self.interval_in_secs.is_empty() {
                // Create a separate interval for file saving
                let log_interval =
                    time::every(Duration::from_secs(interval_secs)).map(|_| Message::LogToFile);

                Subscription::batch([tick_interval, log_interval].into_iter())
            } else {
                // Only monitor without saving
                tick_interval
            }
        } else {
            Subscription::none()
        }
    }
}
