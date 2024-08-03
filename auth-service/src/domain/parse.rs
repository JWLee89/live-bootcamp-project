pub trait Parseable<T, Error> {
    type Output;
    fn parse(input: T) -> Result<Self::Output, Error>;
}
