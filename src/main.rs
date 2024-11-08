mod models;
mod system_monitor;
mod utils;
use iced::Application;
use iced::Settings;
use models::SystemMonitor;

fn main() -> iced::Result {
    SystemMonitor::run(Settings::default())
}
