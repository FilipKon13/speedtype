use runner::Runner;

pub mod runner;

fn main() -> std::io::Result<()> {
    Runner::new().run()
}
