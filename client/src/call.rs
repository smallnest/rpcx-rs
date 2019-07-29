#[derive(Default, Debug)]
pub struct Call<T, U> {
    pub service_path: String,
    pub service_method: String,
    pub seq: u64,
    pub args: T,
    pub reply: U,
    pub error: String,
}

impl<T: Default, U: Default> Call<T, U> {
    pub fn new() -> Self {
        Default::default()
    }
}
