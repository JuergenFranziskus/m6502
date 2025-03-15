#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Core {
    pub a: u8,
    pub p: P,
    pub pc: u16,
    pub s: u8,
    pub x: u8,
    pub y: u8,
}
impl Core {
    pub fn set_flags(&mut self, from: u8) {
        self.p.set_n(from >= 0x80);
        self.p.set_z(from == 0);
    }
    pub fn set_a_flags(&mut self) {
        self.set_flags(self.a);
    }
    pub fn set_x_flags(&mut self) {
        self.set_flags(self.x);
    }
    pub fn set_y_flags(&mut self) {
        self.set_flags(self.y);
    }

    pub fn exec_adc(&mut self, b: u8) {
        let (s, c, v) = adc(self.a, b, self.p.c());
        self.a = s;
        self.set_a_flags();
        self.p.set_c(c);
        self.p.set_v(v);
    }
    pub fn exec_anc(&mut self, b: u8) {
        self.exec_and(b);
        self.p.set_c(self.a & 0x80 != 0);
    }
    pub fn exec_and(&mut self, b: u8) {
        self.a &= b;
        self.set_a_flags();
    }
    pub fn exec_ane(&mut self, b: u8, magic: u8) {
        self.a = (self.a | magic) & self.x & b;
        self.set_a_flags();
    }
    pub fn exec_alr(&mut self, b: u8) {
        self.exec_and(b);
        self.a = self.exec_lsr(self.a);
    }
    pub fn exec_arr(&mut self, b: u8) {
        let and = self.a & b;
        self.p.set_v((and & 0x80 != 0) ^ (and & 0x40 != 0));
        let ror = (and >> 1) | self.p.c() as u8 * 128;
        self.p.set_c(and & 0x80 != 0);
        self.a = ror;
        self.set_a_flags();
    }
    pub fn exec_asl(&mut self, data: u8) -> u8 {
        self.p.set_c(data & 0x80 != 0);
        let data = data << 1;
        self.set_flags(data);
        data
    }
    pub fn exec_bcc(&self) -> bool {
        !self.p.c()
    }
    pub fn exec_bcs(&self) -> bool {
        self.p.c()
    }
    pub fn exec_beq(&self) -> bool {
        self.p.z()
    }
    pub fn exec_bit(&mut self, b: u8) {
        self.p.set_n(b & 0x80 != 0);
        self.p.set_v(b & 0x40 != 0);
        self.p.set_z(self.a & b == 0);
    }
    pub fn exec_bmi(&self) -> bool {
        self.p.n()
    }
    pub fn exec_bne(&self) -> bool {
        !self.p.z()
    }
    pub fn exec_bpl(&self) -> bool {
        !self.p.n()
    }
    pub fn exec_bvc(&self) -> bool {
        !self.p.v()
    }
    pub fn exec_bvs(&self) -> bool {
        self.p.v()
    }
    pub fn exec_clc(&mut self) {
        self.p.set_c(false);
    }
    pub fn exec_cld(&mut self) {
        self.p.set_d(false);
    }
    pub fn exec_cli(&mut self) {
        self.p.set_i(false);
    }
    pub fn exec_clv(&mut self) {
        self.p.set_v(false);
    }
    pub fn exec_cmp(&mut self, b: u8) {
        let (dif, c, _) = sbc(self.a, b, true);
        self.p.set_c(c);
        self.set_flags(dif);
    }
    pub fn exec_cpx(&mut self, b: u8) {
        let (dif, c, _) = sbc(self.x, b, true);
        self.p.set_c(c);
        self.set_flags(dif);
    }
    pub fn exec_cpy(&mut self, b: u8) {
        let (dif, c, _) = sbc(self.y, b, true);
        self.p.set_c(c);
        self.set_flags(dif);
    }
    pub fn exec_dcp(&mut self, data: u8) -> u8 {
        let data = self.exec_dec(data);
        self.exec_cmp(data);
        data
    }
    pub fn exec_dec(&mut self, data: u8) -> u8 {
        let data = data.wrapping_sub(1);
        self.set_flags(data);
        data
    }
    pub fn exec_dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.set_x_flags();
    }
    pub fn exec_dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.set_y_flags();
    }
    pub fn exec_eor(&mut self, b: u8) {
        self.a ^= b;
        self.set_a_flags();
    }
    pub fn exec_inc(&mut self, data: u8) -> u8 {
        let data = data.wrapping_add(1);
        self.set_flags(data);
        data
    }
    pub fn exec_inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.set_x_flags();
    }
    pub fn exec_iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.set_y_flags();
    }
    pub fn exec_isc(&mut self, data: u8) -> u8 {
        let data = self.exec_inc(data);
        self.exec_sbc(data);
        data
    }
    pub fn exec_las(&mut self, data: u8) {
        self.a = data & self.s;
        self.x = self.a;
        self.s = self.a;
        self.set_a_flags();
    }
    pub fn exec_lax(&mut self, data: u8) {
        self.a = data;
        self.x = data;
        self.set_a_flags();
    }
    pub fn exec_lda(&mut self, b: u8) {
        self.a = b;
        self.set_a_flags();
    }
    pub fn exec_ldx(&mut self, b: u8) {
        self.x = b;
        self.set_x_flags();
    }
    pub fn exec_ldy(&mut self, b: u8) {
        self.y = b;
        self.set_y_flags();
    }
    pub fn exec_lsr(&mut self, data: u8) -> u8 {
        self.p.set_c(data & 1 != 0);
        let data = data >> 1;
        self.set_flags(data);
        data
    }
    pub fn exec_lxa(&mut self, data: u8, magic: u8) {
        let data = (self.a | magic) & data;
        self.exec_lax(data);
    }
    pub fn exec_ora(&mut self, b: u8) {
        self.a |= b;
        self.set_a_flags();
    }
    pub fn exec_pla(&mut self, data: u8) {
        self.a = data;
        self.set_a_flags();
    }
    pub fn exec_plp(&mut self, data: u8) {
        self.p = P::from_pull_byte(data);
    }
    pub fn exec_rla(&mut self, b: u8) -> u8 {
        let out = self.exec_rol(b);
        self.exec_and(out);
        out
    }
    pub fn exec_rol(&mut self, data: u8) -> u8 {
        let new_c = data & 0x80 != 0;
        let data = (data << 1) | self.p.c() as u8;
        self.p.set_c(new_c);
        self.set_flags(data);
        data
    }
    pub fn exec_ror(&mut self, data: u8) -> u8 {
        let new_c = data & 0x1 != 0;
        let data = (data >> 1) | self.p.c() as u8 * 128;
        self.p.set_c(new_c);
        self.set_flags(data);
        data
    }
    pub fn exec_rra(&mut self, b: u8) -> u8 {
        let out = self.exec_ror(b);
        self.exec_adc(out);
        out
    }
    pub fn exec_sbc(&mut self, b: u8) {
        let (s, c, v) = sbc(self.a, b, self.p.c());
        self.a = s;
        self.set_a_flags();
        self.p.set_c(c);
        self.p.set_v(v);
    }
    pub fn exec_sec(&mut self) {
        self.p.set_c(true);
    }
    pub fn exec_sed(&mut self) {
        self.p.set_d(true);
    }
    pub fn exec_sei(&mut self) {
        self.p.set_i(true);
    }
    pub fn exec_sbx(&mut self, b: u8) {
        let (dif, c, _) = sbc(self.a & self.x, b, true);
        self.x = dif;
        self.set_x_flags();
        self.p.set_c(c);
    }
    pub fn exec_sha(&mut self, addr: u16, wrap: bool) -> (u8, u16) {
        let hi = (addr >> 8) as u8;
        let out = self.a & self.x & hi.wrapping_add(!wrap as u8);
        let hi = if wrap { out } else { hi };
        let addr = addr & 0xFF;
        let addr = addr | (hi as u16) << 8;
        (out, addr)
    }
    pub fn exec_shx(&mut self, addr: u16, wrap: bool) -> (u8, u16) {
        let hi = (addr >> 8) as u8;
        let out = self.x & hi.wrapping_add(!wrap as u8);
        let hi = if wrap { out } else { hi };
        let addr = addr & 0xFF;
        let addr = addr | (hi as u16) << 8;
        (out, addr)
    }
    pub fn exec_shy(&mut self, addr: u16, wrap: bool) -> (u8, u16) {
        let hi = (addr >> 8) as u8;
        let out = self.y & hi.wrapping_add(!wrap as u8);
        let hi = if wrap { out } else { hi };
        let addr = addr & 0xFF;
        let addr = addr | (hi as u16) << 8;
        (out, addr)
    }
    pub fn exec_slo(&mut self, b: u8) -> u8 {
        let out = self.exec_asl(b);
        self.exec_ora(out);
        out
    }
    pub fn exec_sre(&mut self, b: u8) -> u8 {
        let out = self.exec_lsr(b);
        self.exec_eor(out);
        out
    }
    pub fn exec_tas(&mut self, addr: u16, wrap: bool) -> (u8, u16) {
        self.s = self.a & self.x;
        self.exec_sha(addr, wrap)
    }
    pub fn exec_tax(&mut self) {
        self.x = self.a;
        self.set_x_flags();
    }
    pub fn exec_tay(&mut self) {
        self.y = self.a;
        self.set_y_flags();
    }
    pub fn exec_tsx(&mut self) {
        self.x = self.s;
        self.set_x_flags();
    }
    pub fn exec_txa(&mut self) {
        self.a = self.x;
        self.set_a_flags();
    }
    pub fn exec_txs(&mut self) {
        self.s = self.x;
    }
    pub fn exec_tya(&mut self) {
        self.a = self.y;
        self.set_a_flags();
    }
}

fn adc(a: u8, b: u8, c: bool) -> (u8, bool, bool) {
    let (s0, c0) = a.overflowing_add(b);
    let (s, c1) = s0.overflowing_add(c as u8);
    let c = c0 | c1;
    let v = (a & 0x80 == b & 0x80) && (a & 0x80 != s & 0x80);
    (s, c, v)
}
fn sbc(a: u8, b: u8, c: bool) -> (u8, bool, bool) {
    adc(a, !b, c)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct P(pub u8);
impl P {
    pub fn new() -> Self {
        Self(0).with_o(true)
    }

    pub fn from_pull_byte(b: u8) -> Self {
        P(b).with_o(true).with_b(false)
    }
    pub fn to_push_byte(self, b: bool) -> u8 {
        self.with_b(b).0
    }

    pub fn c(self) -> bool {
        self.0 & Self::C != 0
    }
    pub fn z(self) -> bool {
        self.0 & Self::Z != 0
    }
    pub fn i(self) -> bool {
        self.0 & Self::I != 0
    }
    pub fn d(self) -> bool {
        self.0 & Self::D != 0
    }
    pub fn b(self) -> bool {
        self.0 & Self::B != 0
    }
    pub fn o(self) -> bool {
        self.0 & Self::O != 0
    }
    pub fn v(self) -> bool {
        self.0 & Self::V != 0
    }
    pub fn n(self) -> bool {
        self.0 & Self::N != 0
    }

    pub fn set_c(&mut self, to: bool) {
        self.0 &= !Self::C;
        if to {
            self.0 |= Self::C;
        }
    }
    pub fn set_z(&mut self, to: bool) {
        self.0 &= !Self::Z;
        if to {
            self.0 |= Self::Z;
        }
    }
    pub fn set_i(&mut self, to: bool) {
        self.0 &= !Self::I;
        if to {
            self.0 |= Self::I;
        }
    }
    pub fn set_d(&mut self, to: bool) {
        self.0 &= !Self::D;
        if to {
            self.0 |= Self::D;
        }
    }
    pub fn set_b(&mut self, to: bool) {
        self.0 &= !Self::B;
        if to {
            self.0 |= Self::B;
        }
    }
    pub fn set_o(&mut self, to: bool) {
        self.0 &= !Self::O;
        if to {
            self.0 |= Self::O;
        }
    }
    pub fn set_v(&mut self, to: bool) {
        self.0 &= !Self::V;
        if to {
            self.0 |= Self::V;
        }
    }
    pub fn set_n(&mut self, to: bool) {
        self.0 &= !Self::N;
        if to {
            self.0 |= Self::N;
        }
    }

    pub fn with_c(mut self, to: bool) -> Self {
        self.set_c(to);
        self
    }
    pub fn with_z(mut self, to: bool) -> Self {
        self.set_z(to);
        self
    }
    pub fn with_i(mut self, to: bool) -> Self {
        self.set_i(to);
        self
    }
    pub fn with_d(mut self, to: bool) -> Self {
        self.set_d(to);
        self
    }
    pub fn with_b(mut self, to: bool) -> Self {
        self.set_b(to);
        self
    }
    pub fn with_o(mut self, to: bool) -> Self {
        self.set_o(to);
        self
    }
    pub fn with_v(mut self, to: bool) -> Self {
        self.set_v(to);
        self
    }
    pub fn with_n(mut self, to: bool) -> Self {
        self.set_n(to);
        self
    }

    const C: u8 = 1;
    const Z: u8 = 2;
    const I: u8 = 4;
    const D: u8 = 8;
    const B: u8 = 16;
    const O: u8 = 32;
    const V: u8 = 64;
    const N: u8 = 128;
}
