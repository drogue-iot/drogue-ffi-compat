
mod format;
use core::ffi::c_void;
use crate::printf::format::FormatString;

pub fn vnsprintf(output: &mut [u8], format: &str, args: &[*const c_void]) -> usize {
    let format = FormatString::from(format);
    let formatted = format.merge(output, args);
    formatted.len()
}

#[cfg(test)]
mod tests {
    use core::ffi::c_void;

    #[test]
    fn vnsprintf() {
        let mut output: [u8; 128] = [0; 128];

        super::vnsprintf( &mut output, "Hi there %d", &[
            &42 as *const _ as *const c_void
        ]);

    }
}