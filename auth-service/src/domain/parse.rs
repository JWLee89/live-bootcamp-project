use color_eyre::eyre::Result;

pub trait Parseable<T> {
    fn parse(input: T) -> Result<Self>
    where
        Self: Sized;
}
