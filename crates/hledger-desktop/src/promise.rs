#[derive(Debug)]
pub enum Promise<T> {
    Loading,
    Loaded(T),
}
