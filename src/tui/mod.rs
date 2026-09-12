mod app;
mod ui;
mod model;

use anyhow::Result;
use app::UiApp;

use crate::model::Todos;

pub fn run(todos: Todos) -> Result<()> {
   let mut app = UiApp::new(todos);
   app.run()
}