use core::convert::From;
use core::ops::Shl;

#[derive(Debug, Clone, Copy)]
pub struct CAddr(pub [u8; 8], pub usize);

impl Shl<usize> for CAddr {
    type Output = CAddr;
    fn shl(self, rhs: usize) -> CAddr {
        assert!(rhs == 1);
        CAddr([self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7], 0],
              self.1 - 1)
    }
}

impl From<u8> for CAddr {
    fn from(v: u8) -> CAddr { CAddr([v, 0, 0, 0, 0, 0, 0, 0], 1) }
}

impl From<[u8; 1]> for CAddr {
    fn from(v: [u8; 1]) -> CAddr { CAddr([v[0], 0, 0, 0, 0, 0, 0, 0], 1) }
}

impl From<[u8; 2]> for CAddr {
    fn from(v: [u8; 2]) -> CAddr { CAddr([v[0], v[1], 0, 0, 0, 0, 0, 0], 2) }
}

impl From<[u8; 3]> for CAddr {
    fn from(v: [u8; 3]) -> CAddr { CAddr([v[0], v[1], v[2], 0, 0, 0, 0, 0], 3) }
}

impl From<[u8; 4]> for CAddr {
    fn from(v: [u8; 4]) -> CAddr { CAddr([v[0], v[1], v[2], v[3], 0, 0, 0, 0], 4) }
}

impl From<[u8; 5]> for CAddr {
    fn from(v: [u8; 5]) -> CAddr { CAddr([v[0], v[1], v[2], v[3], v[4], 0, 0, 0], 5) }
}

impl From<[u8; 6]> for CAddr {
    fn from(v: [u8; 6]) -> CAddr { CAddr([v[0], v[1], v[2], v[3], v[4], v[5], 0, 0], 6) }
}

impl From<[u8; 7]> for CAddr {
    fn from(v: [u8; 7]) -> CAddr { CAddr([v[0], v[1], v[2], v[3], v[4], v[5], v[6], 0], 7) }
}

impl From<[u8; 8]> for CAddr {
    fn from(v: [u8; 8]) -> CAddr { CAddr([v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]], 8) }
}
