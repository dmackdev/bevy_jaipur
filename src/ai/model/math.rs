pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    let val = if val > max { max } else { val };
    if val < min {
        min
    } else {
        val
    }
}
