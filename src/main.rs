mod app;
mod assistant;
mod system;

fn main() -> anyhow::Result<()> {
    app::App::run()
}
