use heapless::{
    ArrayLength,
    Vec,
    consts::*,
};
use core::fmt::Result;
use core::fmt::Write;
use crate::atoi::atoi_usize;
use crate::printf::format::Chunk::Literal;

use core::ffi::c_void;
use core::intrinsics::write_bytes;
use crate::variadic::VaList;
use crate::strlen::strlen;
use core::slice::from_raw_parts;

#[derive(Debug)]
pub enum FormatSpec {
    Char,
    Decimal(DecimalFormat),
    ExponentialFloatingPoint,
    FloatingPoint,
    Integer(DecimalFormat),
    Octal(DecimalFormat),
    String,
    UnsignedDecimal(DecimalFormat),
    Hexadecimal(DecimalFormat),
}

struct FormatOutput<'a> {
    output: &'a mut [u8],
    pos: usize,
}

impl<'a> FormatOutput<'a> {
    fn wrap(output: &'a mut [u8]) -> Self {
        FormatOutput {
            output,
            pos: 0,
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> core::result::Result<usize, ()> {
        let mut len_to_write = bytes.len();
        // reserve a space for null terminator
        if len_to_write > self.output.len() - (self.pos + 1) {
            len_to_write = self.output.len() - (self.pos + 1);
        }

        for b in bytes[0..len_to_write].iter() {
            self.output[self.pos] = *b;
            self.pos += 1;
        }

        Ok(len_to_write)
    }

    fn as_bytes(&self) -> &[u8] {
        &self.output[0..self.pos]
    }
}

impl<'a> Write for FormatOutput<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match self.write_bytes(s.as_bytes()) {
            Ok(_) => {
                Ok(())
            }
            Err(_) => {
                Err(core::fmt::Error)
            }
        }
    }
}

impl FormatSpec {
    //fn merge(&self, mut output: &mut FormatOutput, arg_ptr: *const c_void) -> usize {
    fn merge(&self, mut output: &mut FormatOutput, va_list: &mut VaList) -> usize {
        match self {
            FormatSpec::Char => {
                let value: char = va_list.va_arg::<char>();
                core::fmt::write(&mut output, format_args!("{}", value));
            }
            FormatSpec::Decimal(_) => {
                let value: u32 = va_list.va_arg::<u32>();
                core::fmt::write(&mut output, format_args!("{}", value));
            }
            /*
            FormatSpec::ExponentialFloatingPoint => {
                write!( output, "{}", unsafe{ *arg as f32 });
            }
            FormatSpec::FloatingPoint => {
                write!( output, "{}", unsafe{ *arg as f32 });
            }
            FormatSpec::Integer(_) => {
                write!( output, "{}", unsafe{ *arg as u32 });
            }
            */
            FormatSpec::Octal(_) => {
                let value: u32 = va_list.va_arg::<u32>();
                core::fmt::write(&mut output, format_args!("{:o}", value));
            }
            FormatSpec::String => {
                let value_ptr = va_list.va_arg::<*const u8>();
                let len = strlen(value_ptr);
                let slice = unsafe { from_raw_parts(value_ptr, len) };
                output.write_bytes(slice as &[u8]);
            }
            /*
            FormatSpec::UnsignedDecimal(_) => {

            }
            */
            FormatSpec::Hexadecimal(_) => {
                let value: u32 = va_list.va_arg::<u32>();
                core::fmt::write(&mut output, format_args!("{:x}", value));
            }
            _ => {}
        }
        output.pos
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum DecimalFormat {
    Unconstrained,
    SpaceFilled(usize),
    ZeroFilled(usize),
    LeftJustified(usize),
}

impl FormatSpec {
    pub fn from(spec: &[u8]) -> Option<FormatSpec> {
        let c = spec[spec.len() - 1usize] as char;
        let mut format = "".as_bytes();
        if spec.len() != 1 {
            format = &spec[0..spec.len() - 2];
        }
        match c {
            'c' => Some(FormatSpec::Char),
            'd' => Some(FormatSpec::Decimal(Self::parse_simple_number_format(format))),
            'e' => Some(FormatSpec::ExponentialFloatingPoint),
            'f' => Some(FormatSpec::FloatingPoint),
            'i' => Some(FormatSpec::Integer(Self::parse_simple_number_format(format))),
            'o' => Some(FormatSpec::Octal(Self::parse_simple_number_format(format))),
            's' => Some(FormatSpec::String),
            'u' => Some(FormatSpec::UnsignedDecimal(Self::parse_simple_number_format(format))),
            'x' => Some(FormatSpec::Hexadecimal(Self::parse_simple_number_format(format))),
            _ => None,
        }
    }

    fn parse_simple_number_format(fmt: &[u8]) -> DecimalFormat {
        if fmt.len() == 0 {
            return DecimalFormat::Unconstrained;
        }

        let first = fmt[0] as char;

        match first {
            '-' => {
                // left
                let num = atoi_usize(&fmt[1..fmt.len()]);
                if let Some(num) = num {
                    return DecimalFormat::LeftJustified(num);
                }
            }
            '0' => {
                // zero filled
                let num = atoi_usize(&fmt[1..fmt.len()]);
                if let Some(num) = num {
                    return DecimalFormat::ZeroFilled(num);
                }
            }
            _ => {
                let num = atoi_usize(fmt);
                if let Some(num) = num {
                    return DecimalFormat::SpaceFilled(num);
                }
            }
        }

        DecimalFormat::Unconstrained
    }
}

#[derive(Debug)]
pub enum Chunk<'a> {
    Literal(&'a [u8]),
    Format(FormatSpec),
}

#[derive(Debug)]
pub struct FormatString<'a> {
    format: &'a [u8],
    chunks: Vec<Chunk<'a>, U64>,
}

fn is_spec_type(c: u8) -> bool {
    match c {
        b'c' | b'd' | b'e' | b'f' | b'i' | b'o' | b's' | b'u' | b'x' => true,
        _ => false,
    }
}

fn find(slice: &[u8], needle: u8) -> Option<usize> {
    for (index, n) in slice.iter().enumerate() {
        if *n == needle {
            return Some(index);
        }
    }

    None
}

fn find_if(slice: &[u8], searcher: &dyn Fn(u8) -> bool) -> Option<usize> {
    for (index, n) in slice.iter().enumerate() {
        if searcher(*n) {
            return Some(index);
        }
    }

    None
}


impl<'a> FormatString<'a> {
    pub fn from(format: &'a [u8]) -> Self {
        let mut cur = 0;
        let len = format.len();

        let mut chunks = Vec::new();

        let s: &str;

        loop {
            let perc = find(&format[cur..len], b'%');
            match perc {
                None => {
                    chunks.push(Literal(&format[cur..format.len()]));
                    break;
                }
                Some(loc) => {
                    let loc = loc + cur;
                    if loc > cur {
                        // there's a literal gap
                        chunks.push(Literal(&format[cur..loc]));
                    }

                    let spec_type = find_if(&format[loc..len], &is_spec_type);
                    match spec_type {
                        None => {
                            break;
                        }
                        Some(mut spec_loc) => {
                            spec_loc = spec_loc + loc;
                            let spec = FormatSpec::from(&format[loc + 1..spec_loc + 1]);
                            if let Some(spec) = spec {
                                //println!("spec: {:?}", spec);
                                chunks.push(Chunk::Format(spec));
                            }
                            cur = spec_loc + 1;
                        }
                    }
                }
            }
        }

        Self {
            format,
            chunks,
        }
    }

    pub(crate) fn merge<'output>(&self, output: &'output mut [u8], va_list: &mut VaList) -> &'output mut [u8] {
        let mut cur_output_index = 0;
        let len = output.len();
        for format_chunk in self.chunks.iter() {
            let mut output_target = &mut FormatOutput::wrap(&mut output[cur_output_index..len]);
            match format_chunk {
                Literal(s) => {
                    let bytes = s;
                    output_target.write_bytes(bytes);
                    cur_output_index += bytes.len();
                }
                Chunk::Format(spec) => {
                    let len = spec.merge(output_target, va_list);
                    cur_output_index += len;
                }
            }
        }

        let mut ret = &mut output[0..cur_output_index + 1];
        // terminate with a null
        ret[cur_output_index] = 0;
        ret
    }
}

/*
#[cfg(test)]
mod tests {
    use super::{FormatString, FormatSpec, DecimalFormat};
    use core::ffi::c_void;
    use crate::printf::format::FormatOutput;

    #[test]
    fn parse_simple_number_format() {
        let fmt = FormatSpec::parse_simple_number_format("");
        assert_eq!(fmt, DecimalFormat::Unconstrained);

        let fmt = FormatSpec::parse_simple_number_format("32");
        assert_eq!(fmt, DecimalFormat::SpaceFilled(32));

        let fmt = FormatSpec::parse_simple_number_format("032");
        assert_eq!(fmt, DecimalFormat::ZeroFilled(32));

        let fmt = FormatSpec::parse_simple_number_format("-32");
        assert_eq!(fmt, DecimalFormat::LeftJustified(32));
    }

    #[test]
    fn format_spec_merge() {
        let spec = FormatSpec::Decimal(DecimalFormat::Unconstrained);
        let mut output: [u8;128] = [0; 128];
        let mut output = FormatOutput::wrap(&mut output);
        let arg = 42;
        let arg_ptr = &arg;
        spec.merge(&mut output, (arg_ptr as *const _ as *const c_void));
        println!( "{}", core::str::from_utf8( output.as_bytes() ).unwrap() );
        //assert_eq!(2 + 2, 4);
    }

    #[test]
    fn format_string_merge() {
        let fmt: FormatString = FormatString::from("%d howdy [0x%x]");
        let arg1 = 42;
        let arg1_ptr = &arg1;

        let args = [
            arg1_ptr as *const _ as *const c_void,
            arg1_ptr as *const _ as *const c_void,
        ];

        let mut output: [u8;128] = [0; 128];

        //let len = fmt.merge(&mut output, &args);
        let mut output = fmt.merge(&mut output, &args);
        println!( "{}", core::str::from_utf8(&output).unwrap());
        assert_eq!( "42 howdy [0x2a]", core::str::from_utf8(&output).unwrap());
        //println!("----> {:?}", fmt);
        //assert_eq!(2 + 2, 4);
    }
}

 */