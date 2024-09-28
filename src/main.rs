use iced::time;
use iced::widget::Column;
use iced::widget::Container;
use iced::widget::Text;
use iced::{executor, Application, Command, Element, Length, Settings, Subscription};
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};

fn main() -> iced::Result {
    SystemMonitor::run(Settings::default())
}

struct SystemMonitor {
    system: System,
    cpu_usage: f32,
    memory_usage: (u64, u64),
    swap_usage: (u64, u64),
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
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

        let total_swap = system.total_swap();
        let used_swap = system.used_swap();

        let cpu_usage = system.global_cpu_info().cpu_usage();

        (
            SystemMonitor {
                system,
                cpu_usage,
                memory_usage: (used_memory, total_memory),
                swap_usage: (used_swap, total_swap),
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
                self.system.refresh_all();

                self.cpu_usage = self.system.global_cpu_info().cpu_usage();
                self.memory_usage = (self.system.used_memory(), self.system.total_memory());
                self.swap_usage = (self.system.used_swap(), self.system.total_swap());
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let cpu_usage = Text::new(format!("CPU usage: {:.2}%", self.cpu_usage)).size(30);

        let memory_usage = Text::new(format!(
            "Memory usage: {} KB / {} KB",
            self.memory_usage.0, self.memory_usage.1
        ))
        .size(30);

        let swap_usage = Text::new(format!(
            "Swap usage: {} KB / {} KB",
            self.swap_usage.0, self.swap_usage.1
        ))
        .size(30);

        let processor_usage =
            self.system
                .cpus()
                .iter()
                .enumerate()
                .fold(Column::new(), |col, (i, cpu)| {
                    col.push(Text::new(format!(
                        "Processor {}: {:.2}% usage",
                        i + 1,
                        cpu.cpu_usage()
                    )))
                });

        let content = Column::new()
            .spacing(20)
            .push(cpu_usage)
            .push(memory_usage)
            .push(swap_usage)
            .push(processor_usage);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
}
