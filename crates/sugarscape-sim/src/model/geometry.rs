#[inline]
pub(super) fn coord_to_idx(x: u8, y: u8, width: u8) -> usize {
    x as usize + y as usize * width as usize
}

#[inline]
pub(super) fn euclidean_distance(x1: u8, y1: u8, x2: u8, y2: u8) -> f32 {
    ((x1 as f32 - x2 as f32).powi(2) + (y1 as f32 - y2 as f32).powi(2)).sqrt()
}
