use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

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
    pub fctrl: Option<u32>,
    pub fstat: Option<u32>,
    pub ftag: Option<u32>,
    pub fiseg: Option<u32>,
    pub fioff: Option<u32>,
    pub foseg: Option<u32>,
    pub fooff: Option<u32>,
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

    pub fs_base: Option<u64>,
    pub gs_base: Option<u64>,
}
impl Registers {
    // The following sadly assume the endianness in order to only read
    // 10 bits in the st* stuff instead of the full 16.
    #[rustfmt::skip] // formatting can only make this horrible code look worse
    pub fn decode(mut input: &[u8]) -> Self {
        let res = Self {
            rax: Some(input.read_u64::<NativeEndian>().unwrap()),
            rbx: Some(input.read_u64::<NativeEndian>().unwrap()),
            rcx: Some(input.read_u64::<NativeEndian>().unwrap()),
            rdx: Some(input.read_u64::<NativeEndian>().unwrap()),
            rsi: Some(input.read_u64::<NativeEndian>().unwrap()),
            rdi: Some(input.read_u64::<NativeEndian>().unwrap()),
            rbp: Some(input.read_u64::<NativeEndian>().unwrap()),
            rsp: Some(input.read_u64::<NativeEndian>().unwrap()),
            r8: Some(input.read_u64::<NativeEndian>().unwrap()),
            r9: Some(input.read_u64::<NativeEndian>().unwrap()),
            r10: Some(input.read_u64::<NativeEndian>().unwrap()),
            r11: Some(input.read_u64::<NativeEndian>().unwrap()),
            r12: Some(input.read_u64::<NativeEndian>().unwrap()),
            r13: Some(input.read_u64::<NativeEndian>().unwrap()),
            r14: Some(input.read_u64::<NativeEndian>().unwrap()),
            r15: Some(input.read_u64::<NativeEndian>().unwrap()),
            rip: Some(input.read_u64::<NativeEndian>().unwrap()),
            eflags: Some(input.read_u32::<NativeEndian>().unwrap()),
            cs: Some(input.read_u32::<NativeEndian>().unwrap()),
            ss: Some(input.read_u32::<NativeEndian>().unwrap()),
            ds: Some(input.read_u32::<NativeEndian>().unwrap()),
            es: Some(input.read_u32::<NativeEndian>().unwrap()),
            fs: Some(input.read_u32::<NativeEndian>().unwrap()),
            gs: Some(input.read_u32::<NativeEndian>().unwrap()),

            st0: Some(input.read_uint128::<NativeEndian>(10).unwrap()),
            st1: Some(input.read_uint128::<NativeEndian>(10).unwrap()),
            st2: Some(input.read_uint128::<NativeEndian>(10).unwrap()),
            st3: Some(input.read_uint128::<NativeEndian>(10).unwrap()),
            st4: Some(input.read_uint128::<NativeEndian>(10).unwrap()),
            st5: Some(input.read_uint128::<NativeEndian>(10).unwrap()),
            st6: Some(input.read_uint128::<NativeEndian>(10).unwrap()),
            st7: Some(input.read_uint128::<NativeEndian>(10).unwrap()),
            fctrl: Some(input.read_u32::<NativeEndian>().unwrap()),
            fstat: Some(input.read_u32::<NativeEndian>().unwrap()),
            ftag: Some(input.read_u32::<NativeEndian>().unwrap()),
            fiseg: Some(input.read_u32::<NativeEndian>().unwrap()),
            fioff: Some(input.read_u32::<NativeEndian>().unwrap()),
            foseg: Some(input.read_u32::<NativeEndian>().unwrap()),
            fooff: Some(input.read_u32::<NativeEndian>().unwrap()),
            fop: Some(input.read_u32::<NativeEndian>().unwrap()),

            xmm0: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm1: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm2: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm3: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm4: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm5: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm6: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm7: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm8: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm9: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm10: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm11: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm12: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm13: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm14: Some(input.read_u128::<NativeEndian>().unwrap()),
            xmm15: Some(input.read_u128::<NativeEndian>().unwrap()),
            mxcsr: Some(input.read_u32::<NativeEndian>().unwrap()),

            fs_base: Some(input.read_u64::<NativeEndian>().unwrap()),
            gs_base: Some(input.read_u64::<NativeEndian>().unwrap()),
        };
        assert!(input.is_empty(), "Input must be empty after parsing registers");
        res
    }
    #[rustfmt::skip] // formatting can only make this horrible code look worse
    pub fn encode(&self, output: &mut Vec<u8>) {
        output.write_u64::<NativeEndian>(self.rax.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.rbx.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.rcx.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.rdx.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.rsi.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.rdi.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.rbp.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.rsp.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.r8.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.r9.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.r10.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.r11.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.r12.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.r13.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.r14.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.r15.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.rip.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.eflags.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.cs.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.ss.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.ds.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.es.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.fs.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.gs.unwrap_or(0)).unwrap();

        output.write_uint128::<NativeEndian>(self.st0.unwrap_or(0), 10).unwrap();
        output.write_uint128::<NativeEndian>(self.st1.unwrap_or(0), 10).unwrap();
        output.write_uint128::<NativeEndian>(self.st2.unwrap_or(0), 10).unwrap();
        output.write_uint128::<NativeEndian>(self.st3.unwrap_or(0), 10).unwrap();
        output.write_uint128::<NativeEndian>(self.st4.unwrap_or(0), 10).unwrap();
        output.write_uint128::<NativeEndian>(self.st5.unwrap_or(0), 10).unwrap();
        output.write_uint128::<NativeEndian>(self.st6.unwrap_or(0), 10).unwrap();
        output.write_uint128::<NativeEndian>(self.st7.unwrap_or(0), 10).unwrap();
        output.write_u32::<NativeEndian>(self.fctrl.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.fstat.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.ftag.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.fiseg.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.fioff.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.foseg.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.fooff.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.fop.unwrap_or(0)).unwrap();

        output.write_u128::<NativeEndian>(self.xmm0.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm1.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm2.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm3.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm4.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm5.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm6.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm7.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm8.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm9.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm10.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm11.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm12.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm13.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm14.unwrap_or(0)).unwrap();
        output.write_u128::<NativeEndian>(self.xmm15.unwrap_or(0)).unwrap();
        output.write_u32::<NativeEndian>(self.mxcsr.unwrap_or(0)).unwrap();

        output.write_u64::<NativeEndian>(self.fs_base.unwrap_or(0)).unwrap();
        output.write_u64::<NativeEndian>(self.gs_base.unwrap_or(0)).unwrap();
    }
}
