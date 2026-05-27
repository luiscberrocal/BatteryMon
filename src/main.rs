use cosmic::applet;

mod app;
mod battery;
use app::App;

fn main() -> cosmic::iced::Result {
    applet::run::<App>(())
}
