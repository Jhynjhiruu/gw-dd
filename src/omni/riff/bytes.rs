use binrw::{binrw, BinRead, BinWrite};
use human_bytes::human_bytes;
use std::fmt::{Debug, Display};

#[binrw]
#[derive(Clone)]
pub struct HumanBytes<T: BinRead + BinWrite>(pub T)
where
    for<'a> <T as binrw::BinRead>::Args<'a>: Default,
    for<'a> <T as binrw::BinWrite>::Args<'a>: Default;

impl<T> Display for HumanBytes<T>
where
    T: Into<f64> + Clone + BinRead + BinWrite,
    for<'a> <T as binrw::BinRead>::Args<'a>: Default,
    for<'a> <T as binrw::BinWrite>::Args<'a>: Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", human_bytes(self.0.clone()))
    }
}

impl<T> Debug for HumanBytes<T>
where
    T: BinRead + BinWrite,
    for<'a> <T as binrw::BinRead>::Args<'a>: Default,
    for<'a> <T as binrw::BinWrite>::Args<'a>: Default,
    HumanBytes<T>: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}
