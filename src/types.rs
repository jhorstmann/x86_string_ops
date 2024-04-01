mod private {
    pub trait Sealed {}

    impl Sealed for i8 {}
    impl Sealed for u8 {}
    impl Sealed for i16 {}
    impl Sealed for u16 {}
    impl Sealed for i32 {}
    impl Sealed for u32 {}
    impl Sealed for i64 {}
    impl Sealed for u64 {}
    impl Sealed for i128 {}
    impl Sealed for u128 {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

pub trait RegisterType: private::Sealed + Copy + PartialEq {
    fn bitwise_eq(&self, other: &Self) -> bool;
}

impl RegisterType for i8 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for u8 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for i16 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for u16 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for i32 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for u32 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for i64 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for u64 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for i128 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for u128 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl RegisterType for f32 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}
impl RegisterType for f64 {
    fn bitwise_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}
