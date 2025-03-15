use core::{Core, P};

use instr::{Am, Op};


pub mod core;
pub mod instr;
#[cfg(test)]
pub mod tests;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bus {
    pub addr: u16,
    pub data: u8,
    flags: u8,
}
impl Bus {
    pub fn new() -> Self {
        Self {
            addr: 0,
            data: 0,
            flags: 0,
        }
    }

    pub fn read(&mut self, addr: u16) {
        self.addr = addr;
        self.set_rw(true);
        self.set_sync(false);
    }
    pub fn read_sync(&mut self, addr: u16) {
        self.addr = addr;
        self.set_rw(true);
        self.set_sync(true);
    }
    pub fn write(&mut self, addr: u16, data: u8) {
        self.addr = addr;
        self.data = data;
        self.set_rw(false);
        self.set_sync(false);
    }

    pub fn irq(self) -> bool {
        self.flags & Self::IRQ != 0
    }
    pub fn nmi(self) -> bool {
        self.flags & Self::NMI != 0
    }
    pub fn res(self) -> bool {
        self.flags & Self::RES != 0
    }
    pub fn rw(self) -> bool {
        self.flags & Self::RW != 0
    }
    pub fn sync(self) -> bool {
        self.flags & Self::SYNC != 0
    }

    pub fn set_irq(&mut self, to: bool) {
        self.flags &= !Self::IRQ;
        if to {
            self.flags |= Self::IRQ;
        }
    }
    pub fn set_nmi(&mut self, to: bool) {
        self.flags &= !Self::NMI;
        if to {
            self.flags |= Self::NMI;
        }
    }
    pub fn set_res(&mut self, to: bool) {
        self.flags &= !Self::RES;
        if to {
            self.flags |= Self::RES;
        }
    }
    pub fn set_rw(&mut self, to: bool) {
        self.flags &= !Self::RW;
        if to {
            self.flags |= Self::RW;
        }
    }
    pub fn set_sync(&mut self, to: bool) {
        self.flags &= !Self::SYNC;
        if to {
            self.flags |= Self::SYNC;
        }
    }

    const IRQ: u8 = 1;
    const NMI: u8 = 2;
    const RES: u8 = 4;
    const RW: u8 = 8;
    const SYNC: u8 = 16;
}


const UNSTABLE_MAGIC: u8 = 0xEE;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct M6502 {
    core: Core,

    am: Am,
    brk: Brk,
    cycle: u8,
    op: Op,

    addr: u16,
    data: u8,
    wrap: bool,

    irq_scheduled: bool,
    last_nmi: bool,
    nmi_scheduled: bool,
}
impl M6502 {
    pub fn start() -> Self {
        let core = Core {
            a: 0,
            p: P::new(),
            pc: 0,
            s: 0,
            x: 0,
            y: 0,
        };
        Self {
            core,

            am: Am::Implied,
            brk: Brk::Res,
            cycle: 1,
            op: Op::Brk,

            addr: 0,
            data: 0,
            wrap: false,

            irq_scheduled: false,
            last_nmi: false,
            nmi_scheduled: false,
        }
    }
    pub fn new(core: Core) -> Self {
        Self {
            core,

            am: Am::Implied,
            brk: Brk::Brk,
            cycle: 1,
            op: Op::Nop,
            
            addr: 0,
            data: 0,
            wrap: false,

            irq_scheduled: false,
            last_nmi: false,
            nmi_scheduled: false,
        }
    }

    pub fn core(self) -> Core {
        self.core
    }

    pub fn clock(&mut self, bus: &mut Bus) {
        if self.cycle == 0 {
            self.finish_sync(bus);
        }
        self.do_step(bus);
        self.latch_interrupts(bus);
    }
    fn finish_sync(&mut self, bus: &mut Bus) {
        if self.nmi_scheduled {
            self.nmi_scheduled = false;
            self.op = Op::Brk;
            self.am = Am::Implied;
            self.brk = Brk::Nmi;
        } else if self.irq_scheduled && !self.core.p.i() {
            self.op = Op::Brk;
            self.am = Am::Implied;
            self.brk = Brk::Irq;
        } else {
            (self.op, self.am) = instr::decode(bus.data);
            self.brk = Brk::Brk;
            self.core.pc = self.core.pc.wrapping_add(1);
        }
    }
    fn do_step(&mut self, bus: &mut Bus) {
        use Am::*;
        use Op::*;
        match (self.op, self.am) {
            (Brk, Implied) => self.exec_brk(bus),
            (Ora, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_ora, bus),
            (Jam, Implied) => self.exec_jam(bus),
            (Slo, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_slo, bus),
            (Nop, Zero) => self.exec_zero(Self::exec_nop, bus),
            (Ora, Zero) => self.exec_zero(Self::exec_ora, bus),
            (Asl, Zero) => self.exec_zero(Self::exec_asl, bus),
            (Slo, Zero) => self.exec_zero(Self::exec_slo, bus),
            (Php, Implied) => self.exec_push(Self::exec_php, bus),
            (Ora, Immediate) => self.exec_immediate(Self::exec_ora, bus),
            (Asl, Accumulator) => self.exec_accumulator(Self::exec_asl, bus),
            (Anc, Immediate) => self.exec_immediate(Self::exec_anc, bus),
            (Nop, Absolute) => self.exec_absolute(Self::exec_nop, bus),
            (Ora, Absolute) => self.exec_absolute(Self::exec_ora, bus),
            (Asl, Absolute) => self.exec_absolute(Self::exec_asl, bus),
            (Slo, Absolute) => self.exec_absolute(Self::exec_slo, bus),
            (Bpl, Relative) => self.exec_relative(Self::exec_bpl, bus),
            (Ora, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_ora, bus),
            (Slo, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_slo, bus),
            (Nop, ZeroX) => self.exec_zero_x(Self::exec_nop, bus),
            (Ora, ZeroX) => self.exec_zero_x(Self::exec_ora, bus),
            (Asl, ZeroX) => self.exec_zero_x(Self::exec_asl, bus),
            (Slo, ZeroX) => self.exec_zero_x(Self::exec_slo, bus),
            (Clc, Implied) => self.exec_implied(Self::exec_clc, bus),
            (Ora, AbsoluteY) => self.exec_absolute_y(Self::exec_ora, bus),
            (Nop, Implied) => self.exec_implied(Self::exec_nop, bus),
            (Slo, AbsoluteY) => self.exec_absolute_y(Self::exec_slo, bus),
            (Nop, AbsoluteX) => self.exec_absolute_x(Self::exec_nop, bus),
            (Ora, AbsoluteX) => self.exec_absolute_x(Self::exec_ora, bus),
            (Asl, AbsoluteX) => self.exec_absolute_x(Self::exec_asl, bus),
            (Slo, AbsoluteX) => self.exec_absolute_x(Self::exec_slo, bus),
            (Jsr, Absolute) => self.exec_jsr(bus),
            (And, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_and, bus),
            (Rla, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_rla, bus),
            (Bit, Zero) => self.exec_zero(Self::exec_bit, bus),
            (And, Zero) => self.exec_zero(Self::exec_and, bus),
            (Rol, Zero) => self.exec_zero(Self::exec_rol, bus),
            (Rla, Zero) => self.exec_zero(Self::exec_rla, bus),
            (Plp, Implied) => self.exec_pull(Self::exec_plp, bus),
            (And, Immediate) => self.exec_immediate(Self::exec_and, bus),
            (Rol, Accumulator) => self.exec_accumulator(Self::exec_rol, bus),
            (Bit, Absolute) => self.exec_absolute(Self::exec_bit, bus),
            (And, Absolute) => self.exec_absolute(Self::exec_and, bus),
            (Rol, Absolute) => self.exec_absolute(Self::exec_rol, bus),
            (Rla, Absolute) => self.exec_absolute(Self::exec_rla, bus),
            (Bmi, Relative) => self.exec_relative(Self::exec_bmi, bus),
            (And, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_and, bus),
            (Rla, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_rla, bus),
            (And, ZeroX) => self.exec_zero_x(Self::exec_and, bus),
            (Rol, ZeroX) => self.exec_zero_x(Self::exec_rol, bus),
            (Rla, ZeroX) => self.exec_zero_x(Self::exec_rla, bus),
            (Sec, Implied) => self.exec_implied(Self::exec_sec, bus),
            (And, AbsoluteY) => self.exec_absolute_y(Self::exec_and, bus),
            (Rla, AbsoluteY) => self.exec_absolute_y(Self::exec_rla, bus),
            (And, AbsoluteX) => self.exec_absolute_x(Self::exec_and, bus),
            (Rol, AbsoluteX) => self.exec_absolute_x(Self::exec_rol, bus),
            (Rla, AbsoluteX) => self.exec_absolute_x(Self::exec_rla, bus),
            (Rti, Implied) => self.exec_rti(bus),
            (Eor, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_eor, bus),
            (Sre, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_sre, bus),
            (Eor, Zero) => self.exec_zero(Self::exec_eor, bus),
            (Lsr, Zero) => self.exec_zero(Self::exec_lsr, bus),
            (Sre, Zero) => self.exec_zero(Self::exec_sre, bus),
            (Pha, Implied) => self.exec_push(Self::exec_pha, bus),
            (Eor, Immediate) => self.exec_immediate(Self::exec_eor, bus),
            (Lsr, Accumulator) => self.exec_accumulator(Self::exec_lsr, bus),
            (Alr, Immediate) => self.exec_immediate(Self::exec_alr, bus),
            (Jmp, Absolute) => self.exec_absolute(Self::exec_jmp, bus),
            (Eor, Absolute) => self.exec_absolute(Self::exec_eor, bus),
            (Lsr, Absolute) => self.exec_absolute(Self::exec_lsr, bus),
            (Sre, Absolute) => self.exec_absolute(Self::exec_sre, bus),
            (Bvc, Relative) => self.exec_relative(Self::exec_bvc, bus),
            (Eor, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_eor, bus),
            (Sre, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_sre, bus),
            (Eor, ZeroX) => self.exec_zero_x(Self::exec_eor, bus),
            (Lsr, ZeroX) => self.exec_zero_x(Self::exec_lsr, bus),
            (Sre, ZeroX) => self.exec_zero_x(Self::exec_sre, bus),
            (Cli, Implied) => self.exec_implied(Self::exec_cli, bus),
            (Eor, AbsoluteY) => self.exec_absolute_y(Self::exec_eor, bus),
            (Sre, AbsoluteY) => self.exec_absolute_y(Self::exec_sre, bus),
            (Eor, AbsoluteX) => self.exec_absolute_x(Self::exec_eor, bus),
            (Lsr, AbsoluteX) => self.exec_absolute_x(Self::exec_lsr, bus),
            (Sre, AbsoluteX) => self.exec_absolute_x(Self::exec_sre, bus),
            (Rts, Implied) => self.exec_rts(bus),
            (Adc, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_adc, bus),
            (Rra, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_rra, bus),
            (Adc, Zero) => self.exec_zero(Self::exec_adc, bus),
            (Ror, Zero) => self.exec_zero(Self::exec_ror, bus),
            (Rra, Zero) => self.exec_zero(Self::exec_rra, bus),
            (Pla, Implied) => self.exec_pull(Self::exec_pla, bus),
            (Adc, Immediate) => self.exec_immediate(Self::exec_adc, bus),
            (Ror, Accumulator) => self.exec_accumulator(Self::exec_ror, bus),
            (Arr, Immediate) => self.exec_immediate(Self::exec_arr, bus),
            (Jmp, Indirect) => self.exec_indirect(Self::exec_jmp, bus),
            (Adc, Absolute) => self.exec_absolute(Self::exec_adc, bus),
            (Ror, Absolute) => self.exec_absolute(Self::exec_ror, bus),
            (Rra, Absolute) => self.exec_absolute(Self::exec_rra, bus),
            (Bvs, Relative) => self.exec_relative(Self::exec_bvs, bus),
            (Adc, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_adc, bus),
            (Rra, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_rra, bus),
            (Adc, ZeroX) => self.exec_zero_x(Self::exec_adc, bus),
            (Ror, ZeroX) => self.exec_zero_x(Self::exec_ror, bus),
            (Rra, ZeroX) => self.exec_zero_x(Self::exec_rra, bus),
            (Sei, Implied) => self.exec_implied(Self::exec_sei, bus),
            (Adc, AbsoluteY) => self.exec_absolute_y(Self::exec_adc, bus),
            (Rra, AbsoluteY) => self.exec_absolute_y(Self::exec_rra, bus),
            (Adc, AbsoluteX) => self.exec_absolute_x(Self::exec_adc, bus),
            (Ror, AbsoluteX) => self.exec_absolute_x(Self::exec_ror, bus),
            (Rra, AbsoluteX) => self.exec_absolute_x(Self::exec_rra, bus),
            (Nop, Immediate) => self.exec_immediate(Self::exec_nop, bus),
            (Sta, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_sta, bus),
            (Sax, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_sax, bus),
            (Sty, Zero) => self.exec_zero(Self::exec_sty, bus),
            (Sta, Zero) => self.exec_zero(Self::exec_sta, bus),
            (Stx, Zero) => self.exec_zero(Self::exec_stx, bus),
            (Sax, Zero) => self.exec_zero(Self::exec_sax, bus),
            (Dey, Implied) => self.exec_implied(Self::exec_dey, bus),
            (Txa, Implied) => self.exec_implied(Self::exec_txa, bus),
            (Ane, Immediate) => self.exec_immediate(Self::exec_ane, bus),
            (Sty, Absolute) => self.exec_absolute(Self::exec_sty, bus),
            (Sta, Absolute) => self.exec_absolute(Self::exec_sta, bus),
            (Stx, Absolute) => self.exec_absolute(Self::exec_stx, bus),
            (Sax, Absolute) => self.exec_absolute(Self::exec_sax, bus),
            (Bcc, Relative) => self.exec_relative(Self::exec_bcc, bus),
            (Sta, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_sta, bus),
            (Sha, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_sha, bus),
            (Sty, ZeroX) => self.exec_zero_x(Self::exec_sty, bus),
            (Sta, ZeroX) => self.exec_zero_x(Self::exec_sta, bus),
            (Stx, ZeroY) => self.exec_zero_y(Self::exec_stx, bus),
            (Sax, ZeroY) => self.exec_zero_y(Self::exec_sax, bus),
            (Tya, Implied) => self.exec_implied(Self::exec_tya, bus),
            (Sta, AbsoluteY) => self.exec_absolute_y(Self::exec_sta, bus),
            (Txs, Implied) => self.exec_implied(Self::exec_txs, bus),
            (Tas, AbsoluteY) => self.exec_absolute_y(Self::exec_tas, bus),
            (Shy, AbsoluteX) => self.exec_absolute_x(Self::exec_shy, bus),
            (Sta, AbsoluteX) => self.exec_absolute_x(Self::exec_sta, bus),
            (Shx, AbsoluteY) => self.exec_absolute_y(Self::exec_shx, bus),
            (Sha, AbsoluteY) => self.exec_absolute_y(Self::exec_sha, bus),
            (Ldy, Immediate) => self.exec_immediate(Self::exec_ldy, bus),
            (Lda, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_lda, bus),
            (Ldx, Immediate) => self.exec_immediate(Self::exec_ldx, bus),
            (Lax, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_lax, bus),
            (Ldy, Zero) => self.exec_zero(Self::exec_ldy, bus),
            (Lda, Zero) => self.exec_zero(Self::exec_lda, bus),
            (Ldx, Zero) => self.exec_zero(Self::exec_ldx, bus),
            (Lax, Zero) => self.exec_zero(Self::exec_lax, bus),
            (Tay, Implied) => self.exec_implied(Self::exec_tay, bus),
            (Lda, Immediate) => self.exec_immediate(Self::exec_lda, bus),
            (Tax, Implied) => self.exec_implied(Self::exec_tax, bus),
            (Lxa, Immediate) => self.exec_immediate(Self::exec_lxa, bus),
            (Ldy, Absolute) => self.exec_absolute(Self::exec_ldy, bus),
            (Lda, Absolute) => self.exec_absolute(Self::exec_lda, bus),
            (Ldx, Absolute) => self.exec_absolute(Self::exec_ldx, bus),
            (Lax, Absolute) => self.exec_absolute(Self::exec_lax, bus),
            (Bcs, Relative) => self.exec_relative(Self::exec_bcs, bus),
            (Lda, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_lda, bus),
            (Lax, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_lax, bus),
            (Ldy, ZeroX) => self.exec_zero_x(Self::exec_ldy, bus),
            (Lda, ZeroX) => self.exec_zero_x(Self::exec_lda, bus),
            (Ldx, ZeroY) => self.exec_zero_y(Self::exec_ldx, bus),
            (Lax, ZeroY) => self.exec_zero_y(Self::exec_lax, bus),
            (Clv, Implied) => self.exec_implied(Self::exec_clv, bus),
            (Lda, AbsoluteY) => self.exec_absolute_y(Self::exec_lda, bus),
            (Tsx, Implied) => self.exec_implied(Self::exec_tsx, bus),
            (Las, AbsoluteY) => self.exec_absolute_y(Self::exec_las, bus),
            (Ldy, AbsoluteX) => self.exec_absolute_x(Self::exec_ldy, bus),
            (Lda, AbsoluteX) => self.exec_absolute_x(Self::exec_lda, bus),
            (Ldx, AbsoluteY) => self.exec_absolute_y(Self::exec_ldx, bus),
            (Lax, AbsoluteY) => self.exec_absolute_y(Self::exec_lax, bus),
            (Cpy, Immediate) => self.exec_immediate(Self::exec_cpy, bus),
            (Cmp, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_cmp, bus),
            (Dcp, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_dcp, bus),
            (Cpy, Zero) => self.exec_zero(Self::exec_cpy, bus),
            (Cmp, Zero) => self.exec_zero(Self::exec_cmp, bus),
            (Dec, Zero) => self.exec_zero(Self::exec_dec, bus),
            (Dcp, Zero) => self.exec_zero(Self::exec_dcp, bus),
            (Iny, Implied) => self.exec_implied(Self::exec_iny, bus),
            (Cmp, Immediate) => self.exec_immediate(Self::exec_cmp, bus),
            (Dex, Implied) => self.exec_implied(Self::exec_dex, bus),
            (Sbx, Immediate) => self.exec_immediate(Self::exec_sbx, bus),
            (Cpy, Absolute) => self.exec_absolute(Self::exec_cpy, bus),
            (Cmp, Absolute) => self.exec_absolute(Self::exec_cmp, bus),
            (Dec, Absolute) => self.exec_absolute(Self::exec_dec, bus),
            (Dcp, Absolute) => self.exec_absolute(Self::exec_dcp, bus),
            (Bne, Relative) => self.exec_relative(Self::exec_bne, bus),
            (Cmp, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_cmp, bus),
            (Dcp, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_dcp, bus),
            (Cmp, ZeroX) => self.exec_zero_x(Self::exec_cmp, bus),
            (Dec, ZeroX) => self.exec_zero_x(Self::exec_dec, bus),
            (Dcp, ZeroX) => self.exec_zero_x(Self::exec_dcp, bus),
            (Cld, Implied) => self.exec_implied(Self::exec_cld, bus),
            (Cmp, AbsoluteY) => self.exec_absolute_y(Self::exec_cmp, bus),
            (Dcp, AbsoluteY) => self.exec_absolute_y(Self::exec_dcp, bus),
            (Cmp, AbsoluteX) => self.exec_absolute_x(Self::exec_cmp, bus),
            (Dec, AbsoluteX) => self.exec_absolute_x(Self::exec_dec, bus),
            (Dcp, AbsoluteX) => self.exec_absolute_x(Self::exec_dcp, bus),
            (Cpx, Immediate) => self.exec_immediate(Self::exec_cpx, bus),
            (Sbc, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_sbc, bus),
            (Isc, IndexedIndirect) => self.exec_indexed_indirect(Self::exec_isc, bus),
            (Cpx, Zero) => self.exec_zero(Self::exec_cpx, bus),
            (Sbc, Zero) => self.exec_zero(Self::exec_sbc, bus),
            (Inc, Zero) => self.exec_zero(Self::exec_inc, bus),
            (Isc, Zero) => self.exec_zero(Self::exec_isc, bus),
            (Inx, Implied) => self.exec_implied(Self::exec_inx, bus),
            (Sbc, Immediate) => self.exec_immediate(Self::exec_sbc, bus),
            (Cpx, Absolute) => self.exec_absolute(Self::exec_cpx, bus),
            (Sbc, Absolute) => self.exec_absolute(Self::exec_sbc, bus),
            (Inc, Absolute) => self.exec_absolute(Self::exec_inc, bus),
            (Isc, Absolute) => self.exec_absolute(Self::exec_isc, bus),
            (Beq, Relative) => self.exec_relative(Self::exec_beq, bus),
            (Sbc, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_sbc, bus),
            (Isc, IndirectIndexed) => self.exec_indirect_indexed(Self::exec_isc, bus),
            (Sbc, ZeroX) => self.exec_zero_x(Self::exec_sbc, bus),
            (Inc, ZeroX) => self.exec_zero_x(Self::exec_inc, bus),
            (Isc, ZeroX) => self.exec_zero_x(Self::exec_isc, bus),
            (Sed, Implied) => self.exec_implied(Self::exec_sed, bus),
            (Sbc, AbsoluteY) => self.exec_absolute_y(Self::exec_sbc, bus),
            (Isc, AbsoluteY) => self.exec_absolute_y(Self::exec_isc, bus),
            (Sbc, AbsoluteX) => self.exec_absolute_x(Self::exec_sbc, bus),
            (Inc, AbsoluteX) => self.exec_absolute_x(Self::exec_inc, bus),
            (Isc, AbsoluteX) => self.exec_absolute_x(Self::exec_isc, bus),
            _ => unreachable!(),
        }
    }
    fn latch_interrupts(&mut self, bus: &mut Bus) {
        self.irq_scheduled = bus.irq();
        self.nmi_scheduled |= !self.last_nmi && bus.nmi();

        self.last_nmi = bus.nmi();
    }

    fn next(&mut self) {
        self.cycle += 1;
    }
    fn skip(&mut self) {
        self.cycle += 2;
    }
    fn goto(&mut self, cycle: u8) {
        self.cycle = cycle;
    }
    fn fetch(&mut self, bus: &mut Bus) {
        bus.read(self.core.pc);
        self.core.pc = self.core.pc.wrapping_add(1);
    }
    fn push(&mut self, data: u8, bus: &mut Bus) {
        bus.write(self.core.s as u16 | 0x100, data);
        self.core.s = self.core.s.wrapping_sub(1);
    }
    fn push_brk(&mut self, data: u8, bus: &mut Bus) {
        if self.brk.push() {
            self.push(data, bus);
        } else {
            bus.read(self.core.s as u16 | 0x100);
            self.core.s = self.core.s.wrapping_sub(1);
        }
    }
    fn pull(&mut self, bus: &mut Bus) {
        self.core.s = self.core.s.wrapping_add(1);
        bus.read(self.core.s as u16 | 0x100);
    }

    fn sync(&mut self, bus: &mut Bus) {
        bus.read_sync(self.core.pc);
        self.goto(0);
    }

    fn exec_absolute(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                self.fetch(bus);
                self.next();
            }
            1 => {
                self.addr = bus.data as u16;
                self.fetch(bus);
                self.next();
            }
            2 => {
                self.addr |= (bus.data as u16) << 8;
                if self.op.reads_operand() {
                    bus.read(self.addr);
                    self.next();
                } else {
                    op(self);
                    bus.write(self.addr, self.data);
                    self.goto(5);
                }
            }
            3 => {
                self.data = bus.data;
                if self.op.is_rmw() {
                    bus.write(self.addr, self.data);
                    op(self);
                    self.next();
                } else {
                    op(self);
                    if self.op.writes_operand() {
                        bus.write(self.addr, self.data);
                        self.skip();
                    } else {
                        self.sync(bus);
                    }
                }
            }
            4 => {
                bus.write(self.addr, self.data);
                self.next();
            }
            5 => self.sync(bus),
            _ => unreachable!(),
        }
    }
    fn exec_absolute_x(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        self.exec_absolute_index(op, self.core.x, bus);
    }
    fn exec_absolute_y(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        self.exec_absolute_index(op, self.core.y, bus);
    }
    fn exec_accumulator(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                bus.read(self.core.pc);
                self.next();
            }
            1 => {
                self.data = self.core.a;
                op(self);
                self.core.a = self.data;
                self.sync(bus);
            }
            _ => unreachable!(),
        }
    }
    fn exec_immediate(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                self.fetch(bus);
                self.next();
            }
            1 => {
                self.data = bus.data;
                op(self);
                self.sync(bus);
            }
            _ => unreachable!(),
        }
    }
    fn exec_implied(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                bus.read(self.core.pc);
                op(self);
                self.next();
            }
            1 => self.sync(bus),
            _ => unreachable!(),
        }
    }
    fn exec_indexed_indirect(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => { self.fetch(bus); self.next() }
            1 => { bus.read(bus.data as u16); self.data = bus.data; self.next(); }
            2 => {
                self.data = self.data.wrapping_add(self.core.x);
                bus.read(self.data as u16);
                self.next();
            }
            3 => {
                self.addr = bus.data as u16;
                self.data = self.data.wrapping_add(1);
                bus.read(self.data as u16);
                self.next();
            }
            4 => {
                self.addr |= (bus.data as u16) << 8;
                if self.op.reads_operand() {
                    bus.read(self.addr);
                    self.next();
                } else {
                    op(self);
                    bus.write(self.addr, self.data);
                    self.goto(7);
                }
            }
            5 => {
                self.data = bus.data;
                if self.op.is_rmw() {
                    bus.write(self.addr, self.data);
                    op(self);
                    self.next();
                } else {
                    op(self);
                    if self.op.writes_operand() {
                        bus.write(self.addr, self.data);
                        self.skip();
                    } else {
                        self.sync(bus);
                    }
                }
            }
            6 => {
                bus.write(self.addr, self.data);
                self.next();
            }
            7 => self.sync(bus),
            _ => unreachable!(),
        }
    }
    fn exec_indirect(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => { self.fetch(bus); self.next(); }
            1 => { self.addr = bus.data as u16; self.fetch(bus); self.next(); }
            2 => {
                self.addr |= (bus.data as u16) << 8;
                bus.read(self.addr);
                self.next();
            }
            3 => {
                let hi = self.addr & 0xFF00;
                let lo = self.addr.wrapping_add(1) & 0x00FF;
                bus.read(hi | lo);
                self.addr = bus.data as u16;
                self.next();
            }
            4 => {
                self.addr |= (bus.data as u16) << 8;
                op(self);
                self.sync(bus);
            }
            _ => unreachable!(),
        }
    }
    fn exec_indirect_indexed(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                self.fetch(bus);
                self.next();
            }
            1 => {
                self.data = bus.data;
                bus.read(self.data as u16);
                self.next();
            }
            2 => {
                self.addr = bus.data as u16;
                bus.read(self.data.wrapping_add(1) as u16);
                self.next();
            }
            3 => {
                self.addr |= (bus.data as u16) << 8;
                let new = self.addr.wrapping_add(self.core.y as u16);
                let c = self.addr & 0xFF00 != new & 0xFF00;
                let without_wrap = self.addr & 0xFF00 | new & 0x00FF;
                self.addr = new;
                self.wrap = c;
                bus.read(without_wrap);

                let stall = (self.op.reads_operand() && c) || self.op.is_rmw();
                if !stall {
                    self.skip();
                } else {
                    self.next();
                }
            }
            4 => {
                bus.read(self.addr);
                self.next();
            }
            5 => {
                self.data = bus.data;
                if self.op.is_rmw() {
                    bus.write(self.addr, self.data);
                    op(self);
                    self.next();
                } else {
                    op(self);
                    if self.op.writes_operand() {
                        bus.write(self.addr, self.data);
                        self.skip();
                    } else {
                        self.sync(bus);
                    }
                }
            }
            6 => {
                bus.write(self.addr, self.data);
                self.next();
            }
            7 => self.sync(bus),
            _ => unreachable!(),
        }
    }
    fn exec_pull(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                bus.read(self.core.pc);
                self.next();
            }
            1 => {
                bus.read(self.core.s as u16 | 0x100);
                self.next();
            }
            2 => {
                self.pull(bus);
                self.next();
            }
            3 => {
                self.data = bus.data;
                op(self);
                self.sync(bus);
            }
            _ => unreachable!(),
        }
    }
    fn exec_push(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                bus.read(self.core.pc);
                self.next();
            }
            1 => {
                op(self);
                self.push(self.data, bus);
                self.next();
            }
            2 => self.sync(bus),
            _ => unreachable!(),
        }
    }
    fn exec_relative(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                self.fetch(bus);
                self.next();
            }
            1 => {
                let offset = bus.data as i8 as i16;
                let old = self.core.pc;
                let new = old.wrapping_add_signed(offset);
                let wrap = old & 0xFF00 != new & 0xFF00;
                self.addr = new;

                op(self);
                let take = self.data != 0;

                if take && wrap {
                    bus.read(self.core.pc);
                    self.next();
                } else if take {
                    bus.read(self.core.pc);
                    self.skip();
                } else {
                    self.sync(bus);
                }
            }
            2 => {
                let without_wrap = self.core.pc & 0xFF00 | self.addr & 0x00FF;
                bus.read(without_wrap);
                self.next();
            }
            3 => {
                self.core.pc = self.addr;
                self.sync(bus);
            }
            _ => unreachable!(),
        }
    }
    fn exec_zero(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        match self.cycle {
            0 => {
                self.fetch(bus);
                self.next();
            }
            1 => {
                self.addr = bus.data as u16;
                if self.op.reads_operand() {
                    bus.read(self.addr);
                    self.next();
                } else {
                    op(self);
                    bus.write(self.addr, self.data);
                    self.goto(4);
                }
            }
            2 => {
                self.data = bus.data;
                if self.op.is_rmw() {
                    bus.write(self.addr, self.data);
                    op(self);
                    self.next();
                } else {
                    op(self);
                    if self.op.writes_operand() {
                        bus.write(self.addr, self.data);
                        self.skip();
                    } else {
                        self.sync(bus);
                    }
                }
            }
            3 => {
                bus.write(self.addr, self.data);
                self.next();
            }
            4 => self.sync(bus),
            _ => unreachable!(),
        }
    }
    fn exec_zero_x(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        self.exec_zero_index(op, self.core.x, bus);
    }
    fn exec_zero_y(&mut self, op: fn(&mut Self), bus: &mut Bus) {
        self.exec_zero_index(op, self.core.y, bus);
    }

    fn exec_absolute_index(&mut self, op: fn(&mut Self), x: u8, bus: &mut Bus) {
        match self.cycle {
            0 => {
                self.fetch(bus);
                self.next();
            }
            1 => {
                self.addr = bus.data as u16;
                self.fetch(bus);
                self.next();
            }
            2 => {
                self.addr |= (bus.data as u16) << 8;
                let new = self.addr.wrapping_add(x as u16);
                let c = self.addr & 0xFF00 != new & 0xFF00;
                let without_wrap = self.addr & 0xFF00 | new & 0x00FF;
                self.addr = new;
                self.wrap = c;
                bus.read(without_wrap);

                let stall = (self.op.reads_operand() && c) || self.op.is_rmw();
                if !stall {
                    self.skip();
                } else {
                    self.next();
                }
            }
            3 => {
                bus.read(self.addr);
                self.next();
            }
            4 => {
                self.data = bus.data;
                if self.op.is_rmw() {
                    bus.write(self.addr, self.data);
                    op(self);
                    self.next();
                } else {
                    op(self);
                    if self.op.writes_operand() {
                        bus.write(self.addr, self.data);
                        self.skip();
                    } else {
                        self.sync(bus);
                    }
                }
            }
            5 => {
                bus.write(self.addr, self.data);
                self.next();
            }
            6 => self.sync(bus),
            _ => unreachable!(),
        }
    }
    fn exec_zero_index(&mut self, op: fn(&mut Self), x: u8, bus: &mut Bus) {
        match self.cycle {
            0 => {
                self.fetch(bus);
                self.next();
            }
            1 => {
                bus.read(bus.data as u16);
                self.addr = bus.data.wrapping_add(x) as u16;
                self.next();
            }
            2 => {
                if self.op.reads_operand() {
                    bus.read(self.addr);
                    self.next();
                } else {
                    op(self);
                    bus.write(self.addr, self.data);
                    self.goto(5);
                }
            }
            3 => {
                self.data = bus.data;
                op(self);

                if self.op.is_rmw() {
                    bus.write(self.addr, bus.data);
                    self.next();
                } else {
                    self.sync(bus);
                }
            }
            4 => {
                bus.write(self.addr, self.data);
                self.next();
            }
            5 => self.sync(bus),
            _ => unreachable!(),
        }
    }

    fn exec_adc(&mut self) {
        self.core.exec_adc(self.data);
    }
    fn exec_alr(&mut self) {
        self.core.exec_alr(self.data);
    }
    fn exec_anc(&mut self) {
        self.core.exec_anc(self.data);
    }
    fn exec_and(&mut self) {
        self.core.exec_and(self.data);
    }
    fn exec_ane(&mut self) {
        self.core.exec_ane(self.data, UNSTABLE_MAGIC);
    }
    fn exec_arr(&mut self) {
        self.core.exec_arr(self.data);
    }
    fn exec_asl(&mut self) {
        self.data = self.core.exec_asl(self.data);
    }
    fn exec_bcc(&mut self) {
        self.data = self.core.exec_bcc() as u8;
    }
    fn exec_bcs(&mut self) {
        self.data = self.core.exec_bcs() as u8;
    }
    fn exec_beq(&mut self) {
        self.data = self.core.exec_beq() as u8;
    }
    fn exec_bit(&mut self) {
        self.core.exec_bit(self.data);
    }
    fn exec_bmi(&mut self) {
        self.data = self.core.exec_bmi() as u8;
    }
    fn exec_bne(&mut self) {
        self.data = self.core.exec_bne() as u8;
    }
    fn exec_bpl(&mut self) {
        self.data = self.core.exec_bpl() as u8;
    }
    fn exec_brk(&mut self, bus: &mut Bus) {
        match self.cycle {
            0 => {
                if self.brk.skip_id_byte() {
                    self.fetch(bus);
                } else {
                    bus.read(self.core.pc);
                }
                self.next();
            }
            1 => {
                self.push_brk((self.core.pc >> 8) as u8, bus);
                self.next();
            }
            2 => {
                self.push_brk(self.core.pc as u8, bus);
                self.next();
            }
            3 => {
                self.push_brk(self.core.p.to_push_byte(self.brk.b()), bus);
                self.next();
            }
            4 => {
                bus.read(self.brk.vector());
                self.next();
            }
            5 => {
                self.core.pc = bus.data as u16;
                bus.read(self.brk.vector() + 1);
                self.next();
            }
            6 => {
                self.core.pc |= (bus.data as u16) << 8;
                self.core.p.set_i(true);
                self.sync(bus);
            }
            _ => unreachable!(),
        }
    }
    fn exec_bvc(&mut self) {
        self.data = self.core.exec_bvc() as u8;
    }
    fn exec_bvs(&mut self) {
        self.data = self.core.exec_bvs() as u8;
    }
    fn exec_clc(&mut self) {
        self.core.exec_clc();
    }
    fn exec_cld(&mut self) {
        self.core.exec_cld();
    }
    fn exec_cli(&mut self) {
        self.core.exec_cli();
    }
    fn exec_clv(&mut self) {
        self.core.exec_clv();
    }
    fn exec_cmp(&mut self) {
        self.core.exec_cmp(self.data);
    }
    fn exec_cpx(&mut self) {
        self.core.exec_cpx(self.data);
    }
    fn exec_cpy(&mut self) {
        self.core.exec_cpy(self.data);
    }
    fn exec_dcp(&mut self) {
        self.data = self.core.exec_dcp(self.data);
    }
    fn exec_dec(&mut self) {
        self.data = self.core.exec_dec(self.data);
    }
    fn exec_dex(&mut self) {
        self.core.exec_dex();
    }
    fn exec_dey(&mut self) {
        self.core.exec_dey();
    }
    fn exec_eor(&mut self) {
        self.core.exec_eor(self.data);
    }
    fn exec_inc(&mut self) {
        self.data = self.core.exec_inc(self.data);
    }
    fn exec_inx(&mut self) {
        self.core.exec_inx();
    }
    fn exec_iny(&mut self) {
        self.core.exec_iny();
    }
    fn exec_isc(&mut self) {
        self.data = self.core.exec_isc(self.data);
    }
    fn exec_jam(&mut self, bus: &mut Bus) {
        match self.cycle {
            0 => { bus.read(self.core.pc); self.next(); }
            1 => { bus.read(0xFFFF); self.next(); }
            2 => { bus.read(0xFFFE); self.next(); }
            3 => { bus.read(0xFFFE); self.next(); }
            4.. => { bus.read(0xFFFF); self.next(); }
        }
    }
    fn exec_jmp(&mut self) {
        self.core.pc = self.addr;
    }
    fn exec_jsr(&mut self, bus: &mut Bus) {
        match self.cycle {
            0 => {
                self.fetch(bus);
                self.next();
            }
            1 => {
                self.data = bus.data;
                bus.read(self.core.s as u16 | 0x100);
                self.next();
            }
            2 => {
                self.push(self.core.pc.to_le_bytes()[1], bus);
                self.next();
            }
            3 => {
                self.push(self.core.pc.to_le_bytes()[0], bus);
                self.next();
            }
            4 => {
                self.fetch(bus);
                self.next();
            }
            5 => {
                self.core.pc = u16::from_le_bytes([self.data, bus.data]);
                self.sync(bus);
            }
            _ => unreachable!(),
        }
    }
    fn exec_las(&mut self) {
        self.core.exec_las(self.data);
    }
    fn exec_lax(&mut self) {
        self.core.exec_lax(self.data);
    }
    fn exec_lda(&mut self) {
        self.core.exec_lda(self.data);
    }
    fn exec_ldx(&mut self) {
        self.core.exec_ldx(self.data);
    }
    fn exec_ldy(&mut self) {
        self.core.exec_ldy(self.data);
    }
    fn exec_lsr(&mut self) {
        self.data = self.core.exec_lsr(self.data);
    }
    fn exec_lxa(&mut self) {
        self.core.exec_lxa(self.data, UNSTABLE_MAGIC);
    }
    fn exec_nop(&mut self) {}
    fn exec_ora(&mut self) {
        self.core.exec_ora(self.data);
    }
    fn exec_pha(&mut self) {
        self.data = self.core.a;
    }
    fn exec_php(&mut self) {
        self.data = self.core.p.to_push_byte(true);
    }
    fn exec_pla(&mut self) {
        self.core.exec_pla(self.data);
    }
    fn exec_plp(&mut self) {
        self.core.exec_plp(self.data);
    }
    fn exec_rla(&mut self) {
        self.data = self.core.exec_rla(self.data);
    }
    fn exec_rol(&mut self) {
        self.data = self.core.exec_rol(self.data);
    }
    fn exec_ror(&mut self) {
        self.data = self.core.exec_ror(self.data);
    }
    fn exec_rra(&mut self) {
        self.data = self.core.exec_rra(self.data);
    }
    fn exec_rti(&mut self, bus: &mut Bus) {
        match self.cycle {
            0 => {
                bus.read(self.core.pc);
                self.next();
            }
            1 => {
                bus.read(self.core.s as u16 | 0x100);
                self.next();
            }
            2 => {
                self.pull(bus);
                self.next();
            }
            3 => {
                self.core.p = P::from_pull_byte(bus.data);
                self.pull(bus);
                self.next();
            }
            4 => {
                self.core.pc = bus.data as u16;
                self.pull(bus);
                self.next();
            }
            5 => {
                self.core.pc |= (bus.data as u16) << 8;
                self.sync(bus);
            }
            _ => unreachable!(),
        }
    }
    fn exec_rts(&mut self, bus: &mut Bus) {
        match self.cycle {
            0 => {
                bus.read(self.core.pc);
                self.next();
            }
            1 => {
                bus.read(self.core.s as u16 | 0x100);
                self.next();
            }
            2 => {
                self.pull(bus);
                self.next();
            }
            3 => {
                self.data = bus.data;
                self.pull(bus);
                self.next();
            }
            4 => {
                self.core.pc = u16::from_le_bytes([self.data, bus.data]);
                self.fetch(bus);
                self.next();
            }
            5 => self.sync(bus),
            _ => unreachable!(),
        }
    }
    fn exec_sax(&mut self) {
        self.data = self.core.a & self.core.x;
    }
    fn exec_sbc(&mut self) {
        self.core.exec_sbc(self.data);
    }
    fn exec_sbx(&mut self) {
        self.core.exec_sbx(self.data);
    }
    fn exec_sec(&mut self) {
        self.core.exec_sec();
    }
    fn exec_sed(&mut self) {
        self.core.exec_sed();
    }
    fn exec_sei(&mut self) {
        self.core.exec_sei();
    }
    fn exec_sha(&mut self) {
        (self.data, self.addr) = self.core.exec_sha(self.addr, self.wrap);
    }
    fn exec_shx(&mut self) {
        (self.data, self.addr) = self.core.exec_shx(self.addr, self.wrap);
    }
    fn exec_shy(&mut self) {
        (self.data, self.addr) = self.core.exec_shy(self.addr, self.wrap);
    }
    fn exec_slo(&mut self) {
        self.data = self.core.exec_slo(self.data);
    }
    fn exec_sre(&mut self) {
        self.data = self.core.exec_sre(self.data);
    }
    fn exec_sta(&mut self) {
        self.data = self.core.a;
    }
    fn exec_stx(&mut self) {
        self.data = self.core.x;
    }
    fn exec_sty(&mut self) {
        self.data = self.core.y;
    }
    fn exec_tas(&mut self) {
        (self.data, self.addr) = self.core.exec_tas(self.addr, self.wrap);
    }
    fn exec_tax(&mut self) {
        self.core.exec_tax();
    }
    fn exec_tay(&mut self) {
        self.core.exec_tay();
    }
    fn exec_tsx(&mut self) {
        self.core.exec_tsx();
    }
    fn exec_txa(&mut self) {
        self.core.exec_txa();
    }
    fn exec_txs(&mut self) {
        self.core.exec_txs();
    }
    fn exec_tya(&mut self) {
        self.core.exec_tya();
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Brk {
    Brk,
    Irq,
    Nmi,
    Res,
}
impl Brk {
    fn push(self) -> bool {
        !matches!(self, Self::Res)
    }
    fn b(self) -> bool {
        matches!(self, Self::Brk)
    }
    fn vector(self) -> u16 {
        match self {
            Brk::Brk => 0xFFFE,
            Brk::Irq => 0xFFFE,
            Brk::Nmi => 0xFFFA,
            Brk::Res => 0xFFFC,
        }
    }
    fn skip_id_byte(self) -> bool {
        matches!(self, Self::Brk)
    }
}
