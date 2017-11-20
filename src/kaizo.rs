#[derive(Clone, Debug)]
pub struct Kaizo {
    pub screen_name: String,
    pub status_id: u64,
    pub command: String,
}

impl Kaizo {
    pub fn new<S, C>(screen_name: S, status_id: u64, command: C) -> Self
        where S: Into<String>, C: Into<String>
    {
        Kaizo {
            screen_name: screen_name.into(),
            status_id,
            command: command.into(),
        }
    }
}
