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
        if len_to_write > self.output.len() - self.pos {
            len_to_write = self.output.len() - self.pos;
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
            },
            Err(_) => {
                Err(core::fmt::Error)
            },
        }
    }
}

impl FormatSpec {
    fn merge(&self, mut output: &mut FormatOutput, arg_ptr: *const c_void) -> usize {
        match self {
            FormatSpec::Char => {
                let value = unsafe { (arg_ptr as *const char).read() };
                core::fmt::write(&mut output, format_args!("{}", value) );
            }
            FormatSpec::Decimal(_) => {
                let value = unsafe { (arg_ptr as *const u32).read() };
                core::fmt::write(&mut output, format_args!("{}", value) );
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
                let value = unsafe { (arg_ptr as *const u32).read() };
                core::fmt::write(&mut output, format_args!("{:o}", value) );
            }
            /*
            FormatSpec::String => {
            }
            FormatSpec::UnsignedDecimal(_) => {

            }
            */
            FormatSpec::Hexadecimal(_) => {
                let value = unsafe { (arg_ptr as *const u32).read() };
                core::fmt::write(&mut output, format_args!("{:x}", value) );
            }
            _ => { }
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
    pub fn from(spec: &str) -> Option<FormatSpec> {
        let bytes = spec.as_bytes();
        let c = bytes[bytes.len() - 1usize] as char;
        let mut format = "";
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

    fn parse_simple_number_format(fmt: &str) -> DecimalFormat {
        if fmt.len() == 0 {
            return DecimalFormat::Unconstrained;
        }

        let first = fmt.as_bytes()[0] as char;

        match first {
            '-' => {
                // left
                let num = atoi_usize(fmt[1..fmt.len()].as_bytes());
                if let Some(num) = num {
                    return DecimalFormat::LeftJustified(num);
                }
            }
            '0' => {
                // zero filled
                let num = atoi_usize(fmt[1..fmt.len()].as_bytes());
                if let Some(num) = num {
                    return DecimalFormat::ZeroFilled(num);
                }
            }
            _ => {
                let num = atoi_usize(fmt.as_bytes());
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
    Literal(&'a str),
    Format(FormatSpec),
}

#[derive(Debug)]
pub struct FormatString<'a> {
    format: &'a str,
    chunks: Vec<Chunk<'a>, U64>,
}

fn is_spec_type(c: char) -> bool {
    match c {
        'c' | 'd' | 'e' | 'f' | 'i' | 'o' | 's' | 'u' | 'x' => true,
        _ => false,
    }
}

impl<'a> FormatString<'a> {
    pub fn from(format: &'a str) -> Self {
        let mut cur = 0;
        let len = format.len();

        let mut chunks = Vec::new();

        loop {
            let perc = format[cur..len].find('%');
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

                    let spec_type = format[loc..len].find(is_spec_type);
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

    pub(crate) fn merge<'output>(&self, output: &'output mut [u8], args: &[*const c_void]) -> &'output [u8] {
        let mut cur_arg = 0;
        let mut cur_output_index = 0;
        let len = output.len();
        for format_chunk in self.chunks.iter() {
            let mut output_target = &mut FormatOutput::wrap(&mut output[cur_output_index..len]);
            match format_chunk {
                Literal(s) => {
                    let bytes = s.as_bytes();
                    output_target.write_bytes(bytes);
                    cur_output_index += bytes.len();
                }
                Chunk::Format(spec) => {
                    let len = spec.merge(output_target, args[cur_arg]);
                    cur_arg += 1;
                    cur_output_index += len;
                }
            }
        }

        &mut output[0..cur_output_index]
    }
}

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
