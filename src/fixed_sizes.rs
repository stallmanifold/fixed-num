pub trait BitLength {
    fn bit_len() -> usize;
}

#[derive(PartialEq, Eq)]
pub struct Bits128 {}
impl BitLength for Bits128 {
    #[inline]
    fn bit_len() -> usize { 128 }
}

#[derive(PartialEq, Eq)]
pub struct Bits256 {}
impl BitLength for Bits256 {
    #[inline]
    fn bit_len() -> usize { 256 }   
}

#[derive(PartialEq, Eq)]
pub struct Bits384 {}
impl BitLength for Bits384 {
    #[inline]
    fn bit_len() -> usize { 384 }
}

#[derive(PartialEq, Eq)]
pub struct Bits512 {}
impl BitLength for Bits512 {
    #[inline]    
    fn bit_len() -> usize { 512 }
}

#[derive(PartialEq, Eq)]
pub struct Bits768 {}
impl BitLength for Bits768 {
    #[inline]
    fn bit_len() -> usize { 768 }
}

#[derive(PartialEq, Eq)]
pub struct Bits1024 {}
impl BitLength for Bits1024 {
    #[inline]
    fn bit_len() -> usize { 1024 }
}

#[derive(PartialEq, Eq)]
pub struct Bits2048 {}
impl BitLength for Bits2048 {
    #[inline]
    fn bit_len() -> usize { 2048 }
}

#[derive(PartialEq, Eq)]
pub struct Bits4096 {}
impl BitLength for Bits4096 {
    #[inline]
    fn bit_len() -> usize { 4096 }
}

#[derive(PartialEq, Eq)]
pub struct Bits8192 {}
impl BitLength for Bits8192 {
    #[inline]
    fn bit_len() -> usize { 8192 }
}

#[derive(PartialEq, Eq)]
pub struct Bits16384 {}
impl BitLength for Bits16384 {
    #[inline]
    fn bit_len() -> usize { 16384 }
}
