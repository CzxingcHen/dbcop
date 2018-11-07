use ansi_term::Style;
use std::fmt;

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub struct Event {
    pub write: bool,
    pub variable: usize,
    pub value: usize,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub struct Transaction {
    pub events: Vec<Event>,
    pub success: bool,
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr = format!(
            "<{}({}):{:2}>",
            if self.write { 'W' } else { 'R' },
            self.variable,
            self.value
        );
        // write!(
        //     f,
        //     "{}",
        //     if self.success {
        //         repr
        //     } else {
        //         format!("{}", Style::new().strikethrough().paint(repr))
        //     }
        // )
        if !self.success {
            write!(f, "!");
        }
        write!(f, "{}", repr)
    }
}

impl Event {
    pub fn read(var: usize) -> Self {
        Event {
            write: false,
            variable: var,
            value: 0,
            success: false,
        }
    }
    pub fn write(var: usize, val: usize) -> Self {
        Event {
            write: true,
            variable: var,
            value: val,
            success: false,
        }
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr = format!("{:?}", self.events);
        // write!(
        //     f,
        //     "{}",
        //     if self.success {
        //         repr
        //     } else {
        //         format!("{}", Style::new().strikethrough().paint(repr))
        //     }
        // )
        if !self.success {
            write!(f, "!");
        }
        write!(f, "{}", repr)
    }
}
