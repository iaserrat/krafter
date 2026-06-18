pub fn rd_u16(b: &[u8], p: usize, le: bool) -> u16 {
    let a = [b[p], b[p + 1]];
    if le {
        u16::from_le_bytes(a)
    } else {
        u16::from_be_bytes(a)
    }
}

pub fn wr_u16(b: &mut [u8], p: usize, v: u16, le: bool) {
    let a = if le { v.to_le_bytes() } else { v.to_be_bytes() };
    b[p] = a[0];
    b[p + 1] = a[1];
}

pub fn rd_u32(b: &[u8], p: usize, le: bool) -> u32 {
    let a = [b[p], b[p + 1], b[p + 2], b[p + 3]];
    if le {
        u32::from_le_bytes(a)
    } else {
        u32::from_be_bytes(a)
    }
}

pub fn wr_u32(b: &mut [u8], p: usize, v: u32, le: bool) {
    let a = if le { v.to_le_bytes() } else { v.to_be_bytes() };
    b[p..p + 4].copy_from_slice(&a);
}
