mod models;
mod system_monitor;
use models::SystemMonitor;
mod utils;
use iced::Application;
use iced::Settings;

fn main() -> iced::Result {
    SystemMonitor::run(Settings::default())
}
