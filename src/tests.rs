use serde::Deserialize;

use crate::{core::{Core, P}, Bus, M6502};

#[derive(Deserialize)]
struct Test {
    name: String,
    #[serde(alias = "initial")]
    start: State,
    #[serde(alias = "final")]
    end: State,
    cycles: Vec<Cycle>,
}

#[derive(Deserialize)]
struct State {
    a: u8,
    p: u8,
    pc: u16,
    s: u8,
    x: u8,
    y: u8,
    ram: Vec<(u16, u8)>,
}

#[derive(Deserialize)]
struct Cycle(u16, u8, String);


fn run_test(test: &Test, ram: &mut [u8; 65536]) {
    println!("Running test \"{}\"", test.name);

    let mut cpu = prepare_cpu(&test.start);
    let mut bus = Bus::new();
    prepare_ram(&test.start, ram);

    for cycle in &test.cycles {
        cpu.clock(&mut bus);
        if bus.rw() {
            bus.data = ram[bus.addr as usize];
        }
        else {
            ram[bus.addr as usize] = bus.data;
        }

        compare_cycle(cycle, bus);
    }
    cpu.clock(&mut bus);
    

    compare_cpu(&test.end, cpu.core());
    compare_ram(&test.end, ram);
    println!();
}
fn prepare_cpu(start: &State) -> M6502 {
    let core = Core {
        a: start.a,
        p: P(start.p),
        pc: start.pc,
        s: start.s,
        x: start.x,
        y: start.y,
    };

    M6502::new(core)
}
fn prepare_ram(start: &State, ram: &mut [u8; 65536]) {
    for &(addr, value) in &start.ram {
        ram[addr as usize] = value;
    }
}
fn compare_cycle(cycle: &Cycle, bus: Bus) {
    let addr = cycle.0;
    let data = cycle.1;
    let rw = cycle.2 == "read";

    let mut is_err = false;

    if addr != bus.addr {
        is_err = true;
        eprintln!("ADDR should {addr:0>4x}, is {:0>4x}", bus.addr);
    }

    if data != bus.data {
        is_err = true;
        eprintln!("DATA should {data:0>2x}, is {:0>2x}", bus.data);
    }

    if rw != bus.rw() {
        is_err = true;
        eprintln!("RW   should {rw}, is {}", bus.rw());
    }



    if is_err {
        panic!("Bus activity does not match");
    }
}
fn compare_cpu(end: &State, cpu: Core) {
    let mut is_err = false;

    if end.a != cpu.a {
        is_err = true;
        eprintln!("A  should {:0>2x}, is {:0>2x}", end.a, cpu.a);
    }
    if end.p != cpu.p.0 {
        is_err = true;
        eprintln!("P  should {:0>2x}, is {:0>2x}", end.p, cpu.p.0);
    }
    if end.pc != cpu.pc {
        is_err = true;
        eprintln!("PC should {:0>4x}, is {:0>4x}", end.pc, cpu.pc);
    }
    if end.s != cpu.s {
        is_err = true;
        eprintln!("S  should {:0>2x}, is {:0>2x}", end.s, cpu.s);
    }
    if end.x != cpu.x {
        is_err = true;
        eprintln!("X  should {:0>2x}, is {:0>2x}", end.x, cpu.x);
    }
    if end.y != cpu.y {
        is_err = true;
        eprintln!("Y  should {:0>2x}, is {:0>2x}", end.y, cpu.y);
    }


    if is_err {
        panic!("Cpu state does not match");
    }
}
fn compare_ram(end: &State, ram: &[u8; 65536]) {
    let mut is_err = false;
    for &(addr, should) in &end.ram {
        let is = ram[addr as usize];
        if should != is {
            is_err = true;
            eprintln!("{addr:0>4x} should {should:0>2x}, is {is:0>2x}");
        }
    }

    if is_err {
        panic!("Memory does not match");
    }
}


fn run_test_file(path: &str) {
    let src = std::fs::read_to_string(path).unwrap();
    let tests: Vec<Test> = serde_json::de::from_str(&src).unwrap();
    let mut ram = [0; 65536];
    for test in &tests {
        run_test(test, &mut ram);
    }
}

#[test]
fn opcode_00_brk_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/00.json");
    run_test_file(path);
}
#[test]
fn opcode_01_ora_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/01.json");
    run_test_file(path);
}
#[test]
fn opcode_02_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/02.json");
    run_test_file(path);
}
#[test]
fn opcode_03_slo_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/03.json");
    run_test_file(path);
}
#[test]
fn opcode_04_nop_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/04.json");
    run_test_file(path);
}
#[test]
fn opcode_05_ora_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/05.json");
    run_test_file(path);
}
#[test]
fn opcode_06_asl_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/06.json");
    run_test_file(path);
}
#[test]
fn opcode_07_slo_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/07.json");
    run_test_file(path);
}
#[test]
fn opcode_08_php_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/08.json");
    run_test_file(path);
}
#[test]
fn opcode_09_ora_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/09.json");
    run_test_file(path);
}
#[test]
fn opcode_0a_asl_accumulator() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/0a.json");
    run_test_file(path);
}
#[test]
fn opcode_0b_anc_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/0b.json");
    run_test_file(path);
}
#[test]
fn opcode_0c_nop_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/0c.json");
    run_test_file(path);
}
#[test]
fn opcode_0d_ora_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/0d.json");
    run_test_file(path);
}
#[test]
fn opcode_0e_asl_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/0e.json");
    run_test_file(path);
}
#[test]
fn opcode_0f_slo_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/0f.json");
    run_test_file(path);
}
#[test]
fn opcode_10_bpl_relative() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/10.json");
    run_test_file(path);
}
#[test]
fn opcode_11_ora_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/11.json");
    run_test_file(path);
}
#[test]
fn opcode_12_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/12.json");
    run_test_file(path);
}
#[test]
fn opcode_13_slo_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/13.json");
    run_test_file(path);
}
#[test]
fn opcode_14_nop_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/14.json");
    run_test_file(path);
}
#[test]
fn opcode_15_ora_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/15.json");
    run_test_file(path);
}
#[test]
fn opcode_16_asl_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/16.json");
    run_test_file(path);
}
#[test]
fn opcode_17_slo_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/17.json");
    run_test_file(path);
}
#[test]
fn opcode_18_clc_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/18.json");
    run_test_file(path);
}
#[test]
fn opcode_19_ora_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/19.json");
    run_test_file(path);
}
#[test]
fn opcode_1a_nop_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/1a.json");
    run_test_file(path);
}
#[test]
fn opcode_1b_slo_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/1b.json");
    run_test_file(path);
}
#[test]
fn opcode_1c_nop_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/1c.json");
    run_test_file(path);
}
#[test]
fn opcode_1d_ora_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/1d.json");
    run_test_file(path);
}
#[test]
fn opcode_1e_asl_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/1e.json");
    run_test_file(path);
}
#[test]
fn opcode_1f_slo_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/1f.json");
    run_test_file(path);
}
#[test]
fn opcode_20_jsr_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/20.json");
    run_test_file(path);
}
#[test]
fn opcode_21_and_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/21.json");
    run_test_file(path);
}
#[test]
fn opcode_22_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/22.json");
    run_test_file(path);
}
#[test]
fn opcode_23_rla_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/23.json");
    run_test_file(path);
}
#[test]
fn opcode_24_bit_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/24.json");
    run_test_file(path);
}
#[test]
fn opcode_25_and_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/25.json");
    run_test_file(path);
}
#[test]
fn opcode_26_rol_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/26.json");
    run_test_file(path);
}
#[test]
fn opcode_27_rla_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/27.json");
    run_test_file(path);
}
#[test]
fn opcode_28_plp_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/28.json");
    run_test_file(path);
}
#[test]
fn opcode_29_and_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/29.json");
    run_test_file(path);
}
#[test]
fn opcode_2a_rol_accumulator() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/2a.json");
    run_test_file(path);
}
#[test]
fn opcode_2b_anc_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/2b.json");
    run_test_file(path);
}
#[test]
fn opcode_2c_bit_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/2c.json");
    run_test_file(path);
}
#[test]
fn opcode_2d_and_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/2d.json");
    run_test_file(path);
}
#[test]
fn opcode_2e_rol_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/2e.json");
    run_test_file(path);
}
#[test]
fn opcode_2f_rla_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/2f.json");
    run_test_file(path);
}
#[test]
fn opcode_30_bmi_relative() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/30.json");
    run_test_file(path);
}
#[test]
fn opcode_31_and_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/31.json");
    run_test_file(path);
}
#[test]
fn opcode_32_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/32.json");
    run_test_file(path);
}
#[test]
fn opcode_33_rla_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/33.json");
    run_test_file(path);
}
#[test]
fn opcode_34_nop_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/34.json");
    run_test_file(path);
}
#[test]
fn opcode_35_and_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/35.json");
    run_test_file(path);
}
#[test]
fn opcode_36_rol_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/36.json");
    run_test_file(path);
}
#[test]
fn opcode_37_rla_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/37.json");
    run_test_file(path);
}
#[test]
fn opcode_38_sec_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/38.json");
    run_test_file(path);
}
#[test]
fn opcode_39_and_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/39.json");
    run_test_file(path);
}
#[test]
fn opcode_3a_nop_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/3a.json");
    run_test_file(path);
}
#[test]
fn opcode_3b_rla_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/3b.json");
    run_test_file(path);
}
#[test]
fn opcode_3c_nop_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/3c.json");
    run_test_file(path);
}
#[test]
fn opcode_3d_and_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/3d.json");
    run_test_file(path);
}
#[test]
fn opcode_3e_rol_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/3e.json");
    run_test_file(path);
}
#[test]
fn opcode_3f_rla_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/3f.json");
    run_test_file(path);
}
#[test]
fn opcode_40_rti_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/40.json");
    run_test_file(path);
}
#[test]
fn opcode_41_eor_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/41.json");
    run_test_file(path);
}
#[test]
fn opcode_42_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/42.json");
    run_test_file(path);
}
#[test]
fn opcode_43_sre_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/43.json");
    run_test_file(path);
}
#[test]
fn opcode_44_nop_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/44.json");
    run_test_file(path);
}
#[test]
fn opcode_45_eor_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/45.json");
    run_test_file(path);
}
#[test]
fn opcode_46_lsr_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/46.json");
    run_test_file(path);
}
#[test]
fn opcode_47_sre_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/47.json");
    run_test_file(path);
}
#[test]
fn opcode_48_pha_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/48.json");
    run_test_file(path);
}
#[test]
fn opcode_49_eor_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/49.json");
    run_test_file(path);
}
#[test]
fn opcode_4a_lsr_accumulator() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/4a.json");
    run_test_file(path);
}
#[test]
fn opcode_4b_alr_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/4b.json");
    run_test_file(path);
}
#[test]
fn opcode_4c_jmp_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/4c.json");
    run_test_file(path);
}
#[test]
fn opcode_4d_eor_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/4d.json");
    run_test_file(path);
}
#[test]
fn opcode_4e_lsr_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/4e.json");
    run_test_file(path);
}
#[test]
fn opcode_4f_sre_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/4f.json");
    run_test_file(path);
}
#[test]
fn opcode_50_bvc_relative() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/50.json");
    run_test_file(path);
}
#[test]
fn opcode_51_eor_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/51.json");
    run_test_file(path);
}
#[test]
fn opcode_52_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/52.json");
    run_test_file(path);
}
#[test]
fn opcode_53_sre_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/53.json");
    run_test_file(path);
}
#[test]
fn opcode_54_nop_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/54.json");
    run_test_file(path);
}
#[test]
fn opcode_55_eor_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/55.json");
    run_test_file(path);
}
#[test]
fn opcode_56_lsr_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/56.json");
    run_test_file(path);
}
#[test]
fn opcode_57_sre_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/57.json");
    run_test_file(path);
}
#[test]
fn opcode_58_cli_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/58.json");
    run_test_file(path);
}
#[test]
fn opcode_59_eor_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/59.json");
    run_test_file(path);
}
#[test]
fn opcode_5a_nop_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/5a.json");
    run_test_file(path);
}
#[test]
fn opcode_5b_sre_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/5b.json");
    run_test_file(path);
}
#[test]
fn opcode_5c_nop_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/5c.json");
    run_test_file(path);
}
#[test]
fn opcode_5d_eor_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/5d.json");
    run_test_file(path);
}
#[test]
fn opcode_5e_lsr_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/5e.json");
    run_test_file(path);
}
#[test]
fn opcode_5f_sre_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/5f.json");
    run_test_file(path);
}
#[test]
fn opcode_60_rts_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/60.json");
    run_test_file(path);
}
#[test]
fn opcode_61_adc_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/61.json");
    run_test_file(path);
}
#[test]
fn opcode_62_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/62.json");
    run_test_file(path);
}
#[test]
fn opcode_63_rra_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/63.json");
    run_test_file(path);
}
#[test]
fn opcode_64_nop_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/64.json");
    run_test_file(path);
}
#[test]
fn opcode_65_adc_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/65.json");
    run_test_file(path);
}
#[test]
fn opcode_66_ror_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/66.json");
    run_test_file(path);
}
#[test]
fn opcode_67_rra_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/67.json");
    run_test_file(path);
}
#[test]
fn opcode_68_pla_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/68.json");
    run_test_file(path);
}
#[test]
fn opcode_69_adc_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/69.json");
    run_test_file(path);
}
#[test]
fn opcode_6a_ror_accumulator() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/6a.json");
    run_test_file(path);
}
#[test]
fn opcode_6b_arr_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/6b.json");
    run_test_file(path);
}
#[test]
fn opcode_6c_jmp_indirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/6c.json");
    run_test_file(path);
}
#[test]
fn opcode_6d_adc_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/6d.json");
    run_test_file(path);
}
#[test]
fn opcode_6e_ror_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/6e.json");
    run_test_file(path);
}
#[test]
fn opcode_6f_rra_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/6f.json");
    run_test_file(path);
}
#[test]
fn opcode_70_bvs_relative() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/70.json");
    run_test_file(path);
}
#[test]
fn opcode_71_adc_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/71.json");
    run_test_file(path);
}
#[test]
fn opcode_72_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/72.json");
    run_test_file(path);
}
#[test]
fn opcode_73_rra_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/73.json");
    run_test_file(path);
}
#[test]
fn opcode_74_nop_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/74.json");
    run_test_file(path);
}
#[test]
fn opcode_75_adc_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/75.json");
    run_test_file(path);
}
#[test]
fn opcode_76_ror_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/76.json");
    run_test_file(path);
}
#[test]
fn opcode_77_rra_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/77.json");
    run_test_file(path);
}
#[test]
fn opcode_78_sei_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/78.json");
    run_test_file(path);
}
#[test]
fn opcode_79_adc_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/79.json");
    run_test_file(path);
}
#[test]
fn opcode_7a_nop_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/7a.json");
    run_test_file(path);
}
#[test]
fn opcode_7b_rra_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/7b.json");
    run_test_file(path);
}
#[test]
fn opcode_7c_nop_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/7c.json");
    run_test_file(path);
}
#[test]
fn opcode_7d_adc_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/7d.json");
    run_test_file(path);
}
#[test]
fn opcode_7e_ror_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/7e.json");
    run_test_file(path);
}
#[test]
fn opcode_7f_rra_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/7f.json");
    run_test_file(path);
}
#[test]
fn opcode_80_nop_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/80.json");
    run_test_file(path);
}
#[test]
fn opcode_81_sta_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/81.json");
    run_test_file(path);
}
#[test]
fn opcode_82_nop_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/82.json");
    run_test_file(path);
}
#[test]
fn opcode_83_sax_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/83.json");
    run_test_file(path);
}
#[test]
fn opcode_84_sty_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/84.json");
    run_test_file(path);
}
#[test]
fn opcode_85_sta_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/85.json");
    run_test_file(path);
}
#[test]
fn opcode_86_stx_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/86.json");
    run_test_file(path);
}
#[test]
fn opcode_87_sax_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/87.json");
    run_test_file(path);
}
#[test]
fn opcode_88_dey_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/88.json");
    run_test_file(path);
}
#[test]
fn opcode_89_nop_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/89.json");
    run_test_file(path);
}
#[test]
fn opcode_8a_txa_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/8a.json");
    run_test_file(path);
}
#[test]
fn opcode_8b_ane_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/8b.json");
    run_test_file(path);
}
#[test]
fn opcode_8c_sty_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/8c.json");
    run_test_file(path);
}
#[test]
fn opcode_8d_sta_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/8d.json");
    run_test_file(path);
}
#[test]
fn opcode_8e_stx_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/8e.json");
    run_test_file(path);
}
#[test]
fn opcode_8f_sax_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/8f.json");
    run_test_file(path);
}
#[test]
fn opcode_90_bcc_relative() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/90.json");
    run_test_file(path);
}
#[test]
fn opcode_91_sta_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/91.json");
    run_test_file(path);
}
#[test]
fn opcode_92_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/92.json");
    run_test_file(path);
}
#[test]
fn opcode_93_sha_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/93.json");
    run_test_file(path);
}
#[test]
fn opcode_94_sty_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/94.json");
    run_test_file(path);
}
#[test]
fn opcode_95_sta_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/95.json");
    run_test_file(path);
}
#[test]
fn opcode_96_stx_zeroy() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/96.json");
    run_test_file(path);
}
#[test]
fn opcode_97_sax_zeroy() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/97.json");
    run_test_file(path);
}
#[test]
fn opcode_98_tya_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/98.json");
    run_test_file(path);
}
#[test]
fn opcode_99_sta_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/99.json");
    run_test_file(path);
}
#[test]
fn opcode_9a_txs_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/9a.json");
    run_test_file(path);
}
#[test]
fn opcode_9b_tas_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/9b.json");
    run_test_file(path);
}
#[test]
fn opcode_9c_shy_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/9c.json");
    run_test_file(path);
}
#[test]
fn opcode_9d_sta_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/9d.json");
    run_test_file(path);
}
#[test]
fn opcode_9e_shx_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/9e.json");
    run_test_file(path);
}
#[test]
fn opcode_9f_sha_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/9f.json");
    run_test_file(path);
}
#[test]
fn opcode_a0_ldy_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a0.json");
    run_test_file(path);
}
#[test]
fn opcode_a1_lda_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a1.json");
    run_test_file(path);
}
#[test]
fn opcode_a2_ldx_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a2.json");
    run_test_file(path);
}
#[test]
fn opcode_a3_lax_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a3.json");
    run_test_file(path);
}
#[test]
fn opcode_a4_ldy_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a4.json");
    run_test_file(path);
}
#[test]
fn opcode_a5_lda_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a5.json");
    run_test_file(path);
}
#[test]
fn opcode_a6_ldx_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a6.json");
    run_test_file(path);
}
#[test]
fn opcode_a7_lax_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a7.json");
    run_test_file(path);
}
#[test]
fn opcode_a8_tay_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a8.json");
    run_test_file(path);
}
#[test]
fn opcode_a9_lda_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/a9.json");
    run_test_file(path);
}
#[test]
fn opcode_aa_tax_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/aa.json");
    run_test_file(path);
}
#[test]
fn opcode_ab_lxa_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ab.json");
    run_test_file(path);
}
#[test]
fn opcode_ac_ldy_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ac.json");
    run_test_file(path);
}
#[test]
fn opcode_ad_lda_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ad.json");
    run_test_file(path);
}
#[test]
fn opcode_ae_ldx_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ae.json");
    run_test_file(path);
}
#[test]
fn opcode_af_lax_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/af.json");
    run_test_file(path);
}
#[test]
fn opcode_b0_bcs_relative() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b0.json");
    run_test_file(path);
}
#[test]
fn opcode_b1_lda_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b1.json");
    run_test_file(path);
}
#[test]
fn opcode_b2_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b2.json");
    run_test_file(path);
}
#[test]
fn opcode_b3_lax_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b3.json");
    run_test_file(path);
}
#[test]
fn opcode_b4_ldy_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b4.json");
    run_test_file(path);
}
#[test]
fn opcode_b5_lda_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b5.json");
    run_test_file(path);
}
#[test]
fn opcode_b6_ldx_zeroy() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b6.json");
    run_test_file(path);
}
#[test]
fn opcode_b7_lax_zeroy() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b7.json");
    run_test_file(path);
}
#[test]
fn opcode_b8_clv_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b8.json");
    run_test_file(path);
}
#[test]
fn opcode_b9_lda_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/b9.json");
    run_test_file(path);
}
#[test]
fn opcode_ba_tsx_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ba.json");
    run_test_file(path);
}
#[test]
fn opcode_bb_las_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/bb.json");
    run_test_file(path);
}
#[test]
fn opcode_bc_ldy_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/bc.json");
    run_test_file(path);
}
#[test]
fn opcode_bd_lda_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/bd.json");
    run_test_file(path);
}
#[test]
fn opcode_be_ldx_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/be.json");
    run_test_file(path);
}
#[test]
fn opcode_bf_lax_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/bf.json");
    run_test_file(path);
}
#[test]
fn opcode_c0_cpy_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c0.json");
    run_test_file(path);
}
#[test]
fn opcode_c1_cmp_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c1.json");
    run_test_file(path);
}
#[test]
fn opcode_c2_nop_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c2.json");
    run_test_file(path);
}
#[test]
fn opcode_c3_dcp_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c3.json");
    run_test_file(path);
}
#[test]
fn opcode_c4_cpy_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c4.json");
    run_test_file(path);
}
#[test]
fn opcode_c5_cmp_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c5.json");
    run_test_file(path);
}
#[test]
fn opcode_c6_dec_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c6.json");
    run_test_file(path);
}
#[test]
fn opcode_c7_dcp_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c7.json");
    run_test_file(path);
}
#[test]
fn opcode_c8_iny_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c8.json");
    run_test_file(path);
}
#[test]
fn opcode_c9_cmp_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/c9.json");
    run_test_file(path);
}
#[test]
fn opcode_ca_dex_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ca.json");
    run_test_file(path);
}
#[test]
fn opcode_cb_sbx_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/cb.json");
    run_test_file(path);
}
#[test]
fn opcode_cc_cpy_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/cc.json");
    run_test_file(path);
}
#[test]
fn opcode_cd_cmp_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/cd.json");
    run_test_file(path);
}
#[test]
fn opcode_ce_dec_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ce.json");
    run_test_file(path);
}
#[test]
fn opcode_cf_dcp_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/cf.json");
    run_test_file(path);
}
#[test]
fn opcode_d0_bne_relative() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d0.json");
    run_test_file(path);
}
#[test]
fn opcode_d1_cmp_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d1.json");
    run_test_file(path);
}
#[test]
fn opcode_d2_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d2.json");
    run_test_file(path);
}
#[test]
fn opcode_d3_dcp_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d3.json");
    run_test_file(path);
}
#[test]
fn opcode_d4_nop_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d4.json");
    run_test_file(path);
}
#[test]
fn opcode_d5_cmp_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d5.json");
    run_test_file(path);
}
#[test]
fn opcode_d6_dec_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d6.json");
    run_test_file(path);
}
#[test]
fn opcode_d7_dcp_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d7.json");
    run_test_file(path);
}
#[test]
fn opcode_d8_cld_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d8.json");
    run_test_file(path);
}
#[test]
fn opcode_d9_cmp_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/d9.json");
    run_test_file(path);
}
#[test]
fn opcode_da_nop_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/da.json");
    run_test_file(path);
}
#[test]
fn opcode_db_dcp_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/db.json");
    run_test_file(path);
}
#[test]
fn opcode_dc_nop_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/dc.json");
    run_test_file(path);
}
#[test]
fn opcode_dd_cmp_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/dd.json");
    run_test_file(path);
}
#[test]
fn opcode_de_dec_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/de.json");
    run_test_file(path);
}
#[test]
fn opcode_df_dcp_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/df.json");
    run_test_file(path);
}
#[test]
fn opcode_e0_cpx_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e0.json");
    run_test_file(path);
}
#[test]
fn opcode_e1_sbc_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e1.json");
    run_test_file(path);
}
#[test]
fn opcode_e2_nop_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e2.json");
    run_test_file(path);
}
#[test]
fn opcode_e3_isc_indexedindirect() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e3.json");
    run_test_file(path);
}
#[test]
fn opcode_e4_cpx_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e4.json");
    run_test_file(path);
}
#[test]
fn opcode_e5_sbc_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e5.json");
    run_test_file(path);
}
#[test]
fn opcode_e6_inc_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e6.json");
    run_test_file(path);
}
#[test]
fn opcode_e7_isc_zero() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e7.json");
    run_test_file(path);
}
#[test]
fn opcode_e8_inx_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e8.json");
    run_test_file(path);
}
#[test]
fn opcode_e9_sbc_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/e9.json");
    run_test_file(path);
}
#[test]
fn opcode_ea_nop_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ea.json");
    run_test_file(path);
}
#[test]
fn opcode_eb_sbc_immediate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/eb.json");
    run_test_file(path);
}
#[test]
fn opcode_ec_cpx_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ec.json");
    run_test_file(path);
}
#[test]
fn opcode_ed_sbc_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ed.json");
    run_test_file(path);
}
#[test]
fn opcode_ee_inc_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ee.json");
    run_test_file(path);
}
#[test]
fn opcode_ef_isc_absolute() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ef.json");
    run_test_file(path);
}
#[test]
fn opcode_f0_beq_relative() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f0.json");
    run_test_file(path);
}
#[test]
fn opcode_f1_sbc_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f1.json");
    run_test_file(path);
}
#[test]
fn opcode_f2_jam_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f2.json");
    run_test_file(path);
}
#[test]
fn opcode_f3_isc_indirectindexed() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f3.json");
    run_test_file(path);
}
#[test]
fn opcode_f4_nop_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f4.json");
    run_test_file(path);
}
#[test]
fn opcode_f5_sbc_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f5.json");
    run_test_file(path);
}
#[test]
fn opcode_f6_inc_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f6.json");
    run_test_file(path);
}
#[test]
fn opcode_f7_isc_zerox() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f7.json");
    run_test_file(path);
}
#[test]
fn opcode_f8_sed_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f8.json");
    run_test_file(path);
}
#[test]
fn opcode_f9_sbc_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/f9.json");
    run_test_file(path);
}
#[test]
fn opcode_fa_nop_implied() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/fa.json");
    run_test_file(path);
}
#[test]
fn opcode_fb_isc_absolutey() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/fb.json");
    run_test_file(path);
}
#[test]
fn opcode_fc_nop_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/fc.json");
    run_test_file(path);
}
#[test]
fn opcode_fd_sbc_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/fd.json");
    run_test_file(path);
}
#[test]
fn opcode_fe_inc_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/fe.json");
    run_test_file(path);
}
#[test]
fn opcode_ff_isc_absolutex() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/65x02/nes6502/v1/ff.json");
    run_test_file(path);
}
