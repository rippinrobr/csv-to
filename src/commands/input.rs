use std::fmt;

pub enum InputType {
    CSV,
}

impl fmt::Debug for InputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            InputType::CSV => "CSV",
        };
        write!(f, "{:#?}", printable)
    }
}

// TODO: 
#[derive(Debug)]
pub struct Input {
    pub input_type: InputType,
    pub paths: Vec<String>
}
