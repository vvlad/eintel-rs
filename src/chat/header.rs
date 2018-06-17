use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Fields {
    ExpectingSeparator,
    ExpectingField,
    Field(String, String),
    Done,
}

impl Fields {
    fn line(self, line: &str) -> Self {
        match self {
            Fields::ExpectingSeparator if line.starts_with("-") => Fields::ExpectingField,
            Fields::Field(_, _) | Fields::ExpectingField if line.starts_with("-") => Fields::Done,
            Fields::Field(_, _) | Fields::ExpectingField => {
                let tokens = line
                    .splitn(2, ":")
                    .map(|token| token.trim())
                    .collect::<Vec<_>>();
                if tokens.len() != 2 {
                    Fields::ExpectingField
                } else {
                    Fields::Field(tokens[0].to_owned(), tokens[1].to_owned())
                }
            }
            Fields::Done | Fields::ExpectingSeparator => self,
        }
    }

    pub fn from(header: String) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();
        let mut parser = Fields::ExpectingSeparator;

        let header = header
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        for line in header.lines() {
            let state = parser.line(line);
            if let Fields::Field(field, value) = state.clone() {
                map.insert(field, value);
            }

            parser = state;
        }

        return map;
    }
}

pub enum Length {
    ExpectingFF,
    ExpectingFE,
    Expecting5B,
    PositionFound,
}

impl Length {
    fn parse(&self, c: u8) -> Self {
        match c {
            0xff => match self {
                Length::ExpectingFF => Length::ExpectingFE,
                _ => Length::ExpectingFF,
            },
            0xfe => match self {
                Length::ExpectingFE => Length::Expecting5B,
                _ => Length::ExpectingFF,
            },
            0x5b => match self {
                Length::Expecting5B => Length::PositionFound,
                _ => Length::ExpectingFF,
            },
            _ => Length::ExpectingFF,
        }
    }

    fn done(&self) -> bool {
        match self {
            Length::PositionFound => true,
            _ => false,
        }
    }

    pub fn from(buf: &[u8]) -> Option<usize> {
        let mut state = Length::ExpectingFF;
        let pos = buf.iter().position(|&c| {
            state = state.parse(c);
            state.done()
        })?;

        Some(pos - 2)
    }
}
