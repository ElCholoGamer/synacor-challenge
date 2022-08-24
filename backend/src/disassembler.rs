use std::io::{Write, Result};

pub fn disassemble(bin: &[u16], out: &mut impl Write) -> Result<()> {
    let mut pc = 0;

    while pc < bin.len() {
        let assembly = to_assembly_instruction(&mut pc, bin);
        write!(out, "{:04X}    {}", pc, assembly)?;
    }

    Ok(())
}

pub fn to_assembly_instruction(pc: &mut usize, memory: &[u16]) -> String {
    let opcode = memory[*pc];
    let invalid_str = format!("!{:04X}", opcode);

    let (name, param_count) = match opcode {
        0 => ("halt", 0),
        1 => ("set", 2),
        2 => ("push", 1),
        3 => ("pop", 1),
        4 => ("eq", 3),
        5 => ("gt", 3),
        6 => ("jmp", 1),
        7 => ("jt", 2),
        8 => ("jf", 2),
        9 => ("add", 3),
        10 => ("mult", 3),
        11 => ("mod", 3),
        12 => ("and", 3),
        13 => ("or", 3),
        14 => ("not", 2),
        15 => ("rmem", 2),
        16 => ("wmem", 2),
        17 => ("call", 1),
        18 => ("ret", 0),
        19 => ("out", 1),
        20 => ("in", 1),
        21 => ("noop", 0),
        _ => (&*invalid_str, 0),
    };

    *pc += 1;
    let mut out = String::new();

    if param_count == 0 {
        out.push_str(&format!("{}", name));
        return out;
    }

    let param_strings = memory[*pc..*pc + param_count].iter().map(|&val| {
        if opcode == 19 {
            match val {
                0 => "'[NUL]' ".into(),
                10 => "'\\n'".into(),
                _ => format!("'{}'", val as u8 as char),
            }
        } else if val < 0x8000 {
            format!("#{:04X}", val)
        } else if val < 0x8008 {
            format!("({})", val & 0x7)
        } else {
            format!("!{:04X}", val)
        }
    }).collect::<Vec<_>>();

    *pc += param_count;
    out.push_str(&format!("{:6}{}", name, param_strings.join(", ")));
    out
}