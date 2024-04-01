use crate::{rep_cmps, rep_movs, rep_scas, rep_stos, RegisterType};

pub trait SliceExt<T: RegisterType> {
    fn inline_fill(&mut self, value: T);
    fn inline_position(&self, value: T) -> Option<usize>;
    fn inline_copy_from(&mut self, other: &[T]);
    fn inline_mismatch(&self, other: &[T]) -> Option<usize>;
}

impl<T: RegisterType> SliceExt<T> for [T] {
    #[inline]
    fn inline_fill(&mut self, value: T) {
        unsafe { rep_stos(value, self.as_mut_ptr(), self.len()) }
    }

    #[inline]
    fn inline_position(&self, value: T) -> Option<usize> {
        unsafe { rep_scas(self.as_ptr(), value, self.len()) }
    }

    fn inline_copy_from(&mut self, other: &[T]) {
        let len = self.len();
        assert_eq!(len, other.len(), "length mismatch");
        unsafe { rep_movs(other.as_ptr(), self.as_mut_ptr(), len) }
    }

    #[inline]
    fn inline_mismatch(&self, other: &[T]) -> Option<usize> {
        let len = self.len();
        assert_eq!(len, other.len(), "length mismatch");
        unsafe { rep_cmps(self.as_ptr(), other.as_ptr(), len) }
    }
}

#[cfg(test)]
mod tests {
    use crate::SliceExt;

    #[test]
    fn test_fill() {
        let a = &mut [0_u8; 5];
        a.inline_fill(42);
        assert_eq!(a, &[42_u8; 5])
    }
    #[test]
    fn test_position() {
        let a = &[1_u8, 2, 3, 4, 5];
        assert_eq!(a.inline_position(1), Some(0));
        assert_eq!(a.inline_position(2), Some(1));
        assert_eq!(a.inline_position(5), Some(4));
        assert_eq!(a.inline_position(6), None);
    }

    #[test]
    #[should_panic(expected = "length mismatch")]
    fn test_copy_from_panic() {
        let a = &mut [0_u8; 3];
        let b = &[1, 2, 3, 4];
        a.inline_copy_from(b);
    }

    #[test]
    fn test_copy_from() {
        let a = &mut [0_u8; 5];
        let b = &[1, 2, 3, 4, 5];
        a.copy_from_slice(b);
        assert_eq!(a, b)
    }

    #[test]
    #[should_panic(expected = "length mismatch")]
    fn test_mismatch_panic() {
        let a = &mut [1_u8, 2, 3];
        let b = &[1_u8, 2];
        a.inline_mismatch(b);
    }

    #[test]
    fn test_mismatch() {
        let empty: [u8; 0] = [];
        assert_eq!(empty.inline_mismatch(&empty), None);
        assert_eq!([1_u8, 2, 3].inline_mismatch(&[1_u8, 2, 3]), None);
        assert_eq!([1_u8, 2, 3].inline_mismatch(&[2_u8, 2, 3]), Some(0));
        assert_eq!([1_u8, 2, 3].inline_mismatch(&[1_u8, 5, 6]), Some(1));
        assert_eq!([1_u8, 2, 3].inline_mismatch(&[1_u8, 2, 4]), Some(2));
    }
}
