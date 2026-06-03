mod app;
mod ui;

use anyhow::Result;
use app::UiApp;

pub fn run(todos: Vec<crate::model::Todo>) -> Result<()> {
    let mut app = UiApp::new(todos);
    app.run()
}