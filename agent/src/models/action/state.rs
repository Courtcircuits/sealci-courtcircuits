#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    InProgress = 0,
    Completed = 1,
    Failed = 2,
}
