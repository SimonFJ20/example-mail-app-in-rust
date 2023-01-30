use mail_app::MailApp;
use tui::Tui;

mod mail_app;
mod tui;

fn main() {
    Tui::new(MailApp::new()).run();
}
