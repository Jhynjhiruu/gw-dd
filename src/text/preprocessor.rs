use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreprocessorState {
    Expecting,
    Slash,
    SkipLine,
    SkipComment,
    EndComment,
    Directive,
    DirectiveParameter,
    DirectiveString,
}

#[derive(Debug, Error)]
pub enum PreprocessError {
    #[error("Unexpected token at ({1}:{2}): {0:?}")]
    UnexpectedToken(char, usize, usize),

    #[error("Unexpected end state: {0:?}")]
    UnexpectedEndState(PreprocessorState),

    #[error("Unexpected directive at ({1}:{2}): \"{0}\"")]
    UnknownDirective(String, usize, usize),

    #[error("Expected at least one parameter for {0:?} directive at ({1}:{2})")]
    NoParams(Directive, usize, usize),

    #[error("Too many parameters for {0:?} directive at ({1}:{2})")]
    TooManyParameters(Directive, usize, usize),
}

pub struct Preprocessor {
    definitions: HashMap<String, String>,
}

#[derive(Debug)]
pub enum Directive {
    Define,
    Include,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn preprocess(&mut self, file: &str) -> Result<String, PreprocessError> {
        let mut rv = String::new();

        let mut previous_state = PreprocessorState::Expecting;
        let mut state = PreprocessorState::Expecting;

        let mut index = 0;

        let mut line = 0;
        let mut column = 0;

        let chars = file.chars().collect::<Vec<_>>();

        let mut directive_buf = String::new();
        let mut directive = Directive::Define;
        let mut directive_parameter_buf = vec![];
        let mut directive_parameter_delimiter = '"';
        let mut directive_line = 0;
        let mut directive_column = 0;

        fn parse_directive_buf(
            directive_buf: &str,
            directive_line: usize,
            directive_column: usize,
        ) -> Result<Directive, PreprocessError> {
            match directive_buf {
                "define" => Ok(Directive::Define),
                "include" => Ok(Directive::Include),
                _ => Err(PreprocessError::UnknownDirective(
                    directive_buf.to_string(),
                    directive_line,
                    directive_column,
                )),
            }
        }

        'preprocess_loop: while index < chars.len() {
            let c = chars[index];

            if c != '\r' {
                match state {
                    PreprocessorState::Expecting => match c {
                        '/' => {
                            previous_state = state;
                            state = PreprocessorState::Slash;
                        }
                        '#' => {
                            previous_state = state;
                            state = PreprocessorState::Directive;
                            directive_buf = String::new();
                            directive_line = line;
                            directive_column = column;
                        }
                        '\n' => {
                            column = 0;
                            line += 1;
                            index += 1;
                            rv.push(c);
                            continue;
                        }
                        _ => {
                            for (k, v) in &self.definitions {
                                let len = k.len();

                                if index + len < chars.len()
                                    && &String::from_iter(&chars[index..index + len]) == k
                                {
                                    rv += v;
                                    index += len;
                                    continue 'preprocess_loop;
                                }
                            }
                            rv.push(c);
                        }
                    },
                    PreprocessorState::Slash => match c {
                        '/' => {
                            state = PreprocessorState::SkipLine;
                        }
                        '*' => {
                            state = PreprocessorState::SkipComment;
                        }
                        _ => return Err(PreprocessError::UnexpectedToken(c, line, column)),
                    },
                    PreprocessorState::SkipLine => match c {
                        '\n' => {
                            state = previous_state;
                            column = 0;
                            line += 1;
                            index += 1;
                            rv.push(c);
                            continue;
                        }
                        _ => {}
                    },
                    PreprocessorState::SkipComment => match c {
                        '*' => {
                            state = PreprocessorState::EndComment;
                        }
                        _ => {}
                    },
                    PreprocessorState::EndComment => match c {
                        '/' => {
                            state = previous_state;
                        }
                        _ => return Err(PreprocessError::UnexpectedToken(c, line, column)),
                    },
                    PreprocessorState::Directive => match c {
                        ' ' | '\t' => {
                            directive = parse_directive_buf(
                                &directive_buf,
                                directive_line,
                                directive_column,
                            )?;
                            state = PreprocessorState::DirectiveParameter;
                            directive_parameter_buf = vec![String::new()];
                        }
                        '"' | '<' => {
                            directive = parse_directive_buf(
                                &directive_buf,
                                directive_line,
                                directive_column,
                            )?;
                            state = PreprocessorState::DirectiveParameter;
                            directive_parameter_buf = vec![String::new()];
                            continue;
                        }
                        '\n' => return Err(PreprocessError::UnexpectedToken(c, line, column)),
                        _ => {
                            directive_buf.push(c);
                        }
                    },
                    PreprocessorState::DirectiveParameter => match c {
                        '"' | '<' => {
                            directive_parameter_delimiter = c;
                            state = PreprocessorState::DirectiveString;
                            directive_parameter_buf.last_mut().unwrap().push(c);
                        }
                        '\n' => {
                            if directive_parameter_buf.last().unwrap().is_empty() {
                                directive_parameter_buf.pop();
                            }

                            match directive {
                                Directive::Define => match directive_parameter_buf.len() {
                                    1 => {
                                        self.definitions
                                            .insert(directive_parameter_buf[0].clone(), "".into());
                                    }
                                    2 => {
                                        self.definitions.insert(
                                            directive_parameter_buf[0].clone(),
                                            directive_parameter_buf[1].clone(),
                                        );
                                    }
                                    0 => {
                                        return Err(PreprocessError::NoParams(
                                            directive,
                                            directive_line,
                                            directive_column,
                                        ))
                                    }
                                    _ => {
                                        return Err(PreprocessError::TooManyParameters(
                                            directive,
                                            directive_line,
                                            directive_column,
                                        ))
                                    }
                                },
                                Directive::Include => match directive_parameter_buf.len() {
                                    1 => {
                                        println!("include {}", directive_parameter_buf[0])
                                    }
                                    0 => {
                                        return Err(PreprocessError::NoParams(
                                            directive,
                                            directive_line,
                                            directive_column,
                                        ))
                                    }
                                    _ => {
                                        return Err(PreprocessError::TooManyParameters(
                                            directive,
                                            directive_line,
                                            directive_column,
                                        ))
                                    }
                                },
                            }
                            state = previous_state;
                        }
                        ' ' | '\t' => {
                            if !directive_parameter_buf.last().unwrap().is_empty() {
                                directive_parameter_buf.push(String::new());
                            }
                        }
                        _ => {
                            directive_parameter_buf.last_mut().unwrap().push(c);
                        }
                    },
                    PreprocessorState::DirectiveString => match c {
                        _ if c == directive_parameter_delimiter => {
                            directive_parameter_buf.last_mut().unwrap().push(c);
                            state = PreprocessorState::DirectiveParameter;
                            directive_parameter_buf.push(String::new());
                        }
                        '\n' => return Err(PreprocessError::UnexpectedToken(c, line, column)),
                        _ => {
                            directive_parameter_buf.last_mut().unwrap().push(c);
                        }
                    },
                }
            }

            index += 1;
            column += 1;
        }

        match state {
            PreprocessorState::Expecting | PreprocessorState::SkipLine => Ok(rv),
            PreprocessorState::Slash
            | PreprocessorState::SkipComment
            | PreprocessorState::EndComment
            | PreprocessorState::Directive
            | PreprocessorState::DirectiveParameter
            | PreprocessorState::DirectiveString => Err(PreprocessError::UnexpectedEndState(state)),
        }
    }
}
