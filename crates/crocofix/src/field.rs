use core::fmt;

#[derive(fmt::Debug)]
pub struct Field {
    pub tag: u32,
    pub value: String
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.tag, self.value)
    }
}

