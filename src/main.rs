use app::App;
use std::{
    env,
    io::{self},
};
mod app;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Invalid usage.\n{} <filename>", args[0]);
        return Ok(());
    }
    let mut terminal = ratatui::init();
    let app_result = App::run(&mut terminal, args[1].to_string());
    ratatui::restore();
    app_result
}
