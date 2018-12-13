
pub trait App {
    fn run(self) -> Result<(), std::io::Error>;
}