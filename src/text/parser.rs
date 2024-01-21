use std::{cmp::Ordering, collections::BTreeMap};

use chumsky::{extra::ParserExtra, input::SliceInput, prelude::*};

use crate::types::Vec3;

use super::{
    Block, BlockType, Definition, Duration, Function, LoopingMethod, PaletteManagement, RValue,
    SortingId, Statement, Text, Transparency,
};

#[must_use]
pub fn ident<
    'a,
    I: chumsky::input::ValueInput<'a> + chumsky::input::StrInput<'a, C>,
    C: text::Char,
    E: ParserExtra<'a, I>,
>() -> impl Parser<'a, I, &'a C::Str, E> + Copy {
    any()
        .try_map(|c: C, span| {
            if c.is_ident_start() || /* need underscores at the start to be allowed */ c.to_char() == '_' {
                Ok(c)
            } else {
                Err(chumsky::error::Error::expected_found(
                    [],
                    Some(chumsky::util::MaybeRef::Val(c)),
                    span,
                ))
            }
        })
        .then(select! { c if (c as C).is_ident_continue() => () }.repeated())
        .to_slice()
}

fn integer<'a>() -> impl Parser<'a, &'a str, i32, extra::Err<Rich<'a, char>>> {
    just('-')
        .or_not()
        .then(text::int(10))
        .to_slice()
        .map(|num: &str| num.parse().unwrap())
}

fn float<'a>() -> impl Parser<'a, &'a str, f64, extra::Err<Rich<'a, char>>> {
    let digits = text::digits(10).to_slice();

    let frac = just('.').then(digits);

    let exp = just('e')
        .or(just('E'))
        .then(one_of("+-").or_not())
        .then(digits);

    integer()
        .then(frac.or_not())
        .then(exp.or_not())
        .to_slice()
        .map(|s: &str| s.parse().unwrap())
        .boxed()
}

fn string<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> {
    none_of("\"")
        .repeated()
        .to_slice()
        .delimited_by(just('"'), just('"'))
        .map(str::to_string)
}

impl Vec3 {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        let separator = just(',').padded();

        float()
            .then_ignore(separator)
            .then(float())
            .then_ignore(separator)
            .then(float())
            .delimited_by(just('(').padded(), just(')'))
            .map(|((x, y), z)| Vec3::new(x, y, z))
    }
}

impl LoopingMethod {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        choice((
            just("CACHE").to(Self::Cache),
            just("NONE").to(Self::None),
            just("STREAM").to(Self::Stream),
        ))
    }
}

impl Duration {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        choice((just("INDEFINITE").to(-1), integer())).map(Self)
    }
}

impl PaletteManagement {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        choice((just("NONE").to(Self::None),))
    }
}

impl Transparency {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        choice((just("YES").to(Self::Yes), just("FAST").to(Self::Fast)))
    }
}

impl Definition {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        choice((
            LoopingMethod::parser().map(Self::LoopingMethod),
            Duration::parser().map(Self::Duration),
            PaletteManagement::parser().map(Self::PaletteManagement),
            Transparency::parser().map(Self::Transparency),
        ))
    }
}

impl Function {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        ident()
            .padded()
            .then(
                string()
                    .padded()
                    .or_not()
                    .then(
                        just(',')
                            .padded()
                            .ignored()
                            .then(string().padded())
                            .map(|(_, v)| v)
                            .repeated()
                            .collect::<Vec<_>>(),
                    )
                    .delimited_by(just('('), just(')'))
                    .map(|(first, rest)| {
                        let mut args = match first {
                            Some(f) => vec![f],
                            None => vec![],
                        };
                        args.extend(rest);
                        args
                    }),
            )
            .map(|(name, args)| Function {
                name: name.to_string(),
                args,
            })
    }
}

impl RValue {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        choice((
            string().map(Self::String),
            integer().map(Self::Integer),
            Vec3::parser().map(Self::Vec3),
            Definition::parser().map(Self::Definition),
            Function::parser().map(Self::Function),
        ))
    }
}

fn assignment<'a>() -> impl Parser<'a, &'a str, Statement, extra::Err<Rich<'a, char>>> {
    ident()
        .padded()
        .then_ignore(just('=').padded())
        .then(RValue::parser().padded())
        .then_ignore(just(';'))
        .map(|(i, r)| Statement::Assignment(i.to_string(), r))
}

fn declaration<'a>() -> impl Parser<'a, &'a str, Statement, extra::Err<Rich<'a, char>>> {
    ident()
        .padded()
        .then_ignore(just(';'))
        .map(|i: &str| Statement::Declaration(i.to_string()))
}

impl Statement {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        choice((assignment(), declaration()))
        //assignment()
    }
}

impl BlockType {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        choice((
            just("defineSettings").to(Self::DefineSettings),
            just("defineObject").to(Self::DefineObject),
            just("defineSound").to(Self::DefineSound),
            just("defineEvent").to(Self::DefineEvent),
            just("defineAnim").to(Self::DefineAnim),
            just("parallelAction").to(Self::ParallelAction),
            just("defineStill").to(Self::DefineStill),
            just("serialAction").to(Self::SerialAction),
        ))
    }
}

impl Block {
    fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        BlockType::parser()
            .padded()
            .then(ident().padded())
            .then(just("Weave").padded().or_not())
            .then(
                Statement::parser()
                    .padded()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}')),
            )
            .map(|(((t, n), w), s)| Block {
                id: 0,
                block_type: t,
                name: n.to_string(),
                is_weave: w.is_some(),
                statements: s,
            })
    }
}

impl Text {
    pub fn parser<'a>() -> impl Parser<'a, &'a str, Self, extra::Err<Rich<'a, char>>> {
        Block::parser()
            .padded()
            .repeated()
            .collect::<Vec<_>>()
            .map(|mut blocks| {
                blocks.sort_by(|a, _| {
                    if matches!(a.block_type, BlockType::DefineSettings) {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                });
                let settings = blocks.pop().unwrap();
                Self {
                    settings,
                    blocks: BTreeMap::from_iter(blocks.into_iter().enumerate().map(
                        |(index, elem)| {
                            (
                                SortingId::from_id_index(elem.block_type, 0, &[], index, 0, 0),
                                elem,
                            )
                        },
                    )),
                }
            })
    }
}
