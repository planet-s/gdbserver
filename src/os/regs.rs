use crate::Result;

use std::io::prelude::*;

#[derive(Default)]
pub struct Registers {
    pub rax: Option<u64>,
    pub rbx: Option<u64>,
    pub rcx: Option<u64>,
    pub rdx: Option<u64>,
    pub rsi: Option<u64>,
    pub rdi: Option<u64>,
    pub rbp: Option<u64>,
    pub rsp: Option<u64>,
    pub r8: Option<u64>,
    pub r9: Option<u64>,
    pub r10: Option<u64>,
    pub r11: Option<u64>,
    pub r12: Option<u64>,
    pub r13: Option<u64>,
    pub r14: Option<u64>,
    pub r15: Option<u64>,
    pub rip: Option<u64>,
    pub eflags: Option<u32>,
    pub cs: Option<u32>,
    pub ss: Option<u32>,
    pub ds: Option<u32>,
    pub es: Option<u32>,
    pub fs: Option<u32>,
    pub gs: Option<u32>,
    pub st0: Option<u128>,
    pub st1: Option<u128>,
    pub st2: Option<u128>,
    pub st3: Option<u128>,
    pub st4: Option<u128>,
    pub st5: Option<u128>,
    pub st6: Option<u128>,
    pub st7: Option<u128>,
    pub fcw: Option<u32>,
    pub fstat: Option<u32>,
    pub ftag: Option<u32>,
    pub fiseg: Option<u32>,
    pub fiofs: Option<u32>,
    pub foseg: Option<u32>,
    pub foofs: Option<u32>,
    pub fop: Option<u32>,
    pub xmm0: Option<u128>,
    pub xmm1: Option<u128>,
    pub xmm2: Option<u128>,
    pub xmm3: Option<u128>,
    pub xmm4: Option<u128>,
    pub xmm5: Option<u128>,
    pub xmm6: Option<u128>,
    pub xmm7: Option<u128>,
    pub xmm8: Option<u128>,
    pub xmm9: Option<u128>,
    pub xmm10: Option<u128>,
    pub xmm11: Option<u128>,
    pub xmm12: Option<u128>,
    pub xmm13: Option<u128>,
    pub xmm14: Option<u128>,
    pub xmm15: Option<u128>,
    pub mxcsr: Option<u32>,
}
impl Registers {
    // The following sadly assume the endianness in order to only read
    // 10 bits in the st* stuff instead of the full 16.
    #[rustfmt::skip] // formatting can only make this horrible code look worse
    pub fn decode(mut input: &[u8]) -> Result<Self> {
        let mut byte = || -> Result<u8> {
            let hex = input.get(..2).ok_or("Unexpected EOF decoding registers")?;
            input = &input[2..];
            let hex = std::str::from_utf8(hex)?;
            Ok(u8::from_str_radix(hex, 16)?)
        };
        Ok(Self {
            rax: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            rbx: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            rcx: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            rdx: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            rsi: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            rdi: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            rbp: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            rsp: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            r8: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            r9: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            r10: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            r11: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            r12: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            r13: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            r14: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            r15: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            rip: Some(u64::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            eflags: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            cs: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            ss: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            ds: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            es: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            fs: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            gs: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            st0: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, 0, 0, 0, 0, 0, 0])),
            st1: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, 0, 0, 0, 0, 0, 0])),
            st2: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, 0, 0, 0, 0, 0, 0])),
            st3: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, 0, 0, 0, 0, 0, 0])),
            st4: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, 0, 0, 0, 0, 0, 0])),
            st5: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, 0, 0, 0, 0, 0, 0])),
            st6: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, 0, 0, 0, 0, 0, 0])),
            st7: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, 0, 0, 0, 0, 0, 0])),
            fcw: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            fstat: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            ftag: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            fiseg: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            fiofs: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            foseg: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            foofs: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            fop: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
            xmm0: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm1: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm2: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm3: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm4: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm5: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm6: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm7: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm8: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm9: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm10: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm11: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm12: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm13: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm14: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            xmm15: Some(u128::from_le_bytes([byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?, byte()?])),
            mxcsr: Some(u32::from_le_bytes([byte()?, byte()?, byte()?, byte()?])),
        })
    }
    #[rustfmt::skip] // formatting can only make this horrible code look worse
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<()> {
        let mut write = |slice: Option<&[u8]>, len: usize| {
            if let Some(slice) = slice {
                assert_eq!(slice.len(), len);
                output.reserve(slice.len() * 2);
                for byte in slice {
                    write!(output, "{:02X}", byte).unwrap();
                }
            } else {
                output.reserve(len * 2);
                for _ in 0..len {
                    output.push(b'x');
                    output.push(b'x');
                }
            }
        };
        write(self.rax.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.rbx.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.rcx.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.rdx.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.rsi.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.rdi.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.rbp.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.rsp.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.r8.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.r9.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.r10.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.r11.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.r12.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.r13.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.r14.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.r15.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.rip.map(u64::to_le_bytes).as_ref().map(|s| &s[..]), 8);
        write(self.eflags.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.cs.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.ss.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.ds.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.es.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.fs.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.gs.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.st0.map(u128::to_le_bytes).as_ref().map(|s| &s[..10]), 10);
        write(self.st1.map(u128::to_le_bytes).as_ref().map(|s| &s[..10]), 10);
        write(self.st2.map(u128::to_le_bytes).as_ref().map(|s| &s[..10]), 10);
        write(self.st3.map(u128::to_le_bytes).as_ref().map(|s| &s[..10]), 10);
        write(self.st4.map(u128::to_le_bytes).as_ref().map(|s| &s[..10]), 10);
        write(self.st5.map(u128::to_le_bytes).as_ref().map(|s| &s[..10]), 10);
        write(self.st6.map(u128::to_le_bytes).as_ref().map(|s| &s[..10]), 10);
        write(self.st7.map(u128::to_le_bytes).as_ref().map(|s| &s[..10]), 10);
        write(self.fcw.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.fstat.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.ftag.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.fiseg.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.fiofs.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.foseg.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.foofs.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.fop.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        write(self.xmm0.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm1.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm2.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm3.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm4.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm5.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm6.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm7.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm8.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm9.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm10.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm11.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm12.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm13.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm14.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.xmm15.map(u128::to_le_bytes).as_ref().map(|s| &s[..]), 16);
        write(self.mxcsr.map(u32::to_le_bytes).as_ref().map(|s| &s[..]), 4);
        Ok(())
    }
}
