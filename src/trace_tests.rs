// A trace test is used to compare the result of the model
// with that of a log generated from a real chip in the Chiplab.
// 
// The model passes the test if, after reset, the model produces
// the same signals on each of its output pins as the real chip.
//
// The 6502 chiplab can be found at: https://chiplab.emulationonline.com/6502/
use std::collections::HashMap;
use crate::{W6502, Inputs};

type TraceKV = HashMap<String, String>;

// The trace may have several keys within the signed data, but before
// the trace.
// After the kv data comes the actual trace, which should start with "a="
// Returns the parsed kv data, and the trace that follows.
fn get_trace_kv(mut trace_content: &str) -> Result<(TraceKV, &str/*rest*/), String> {
    const ALLOWED_KEYS : &[&'static str] = &[
        "InputSha256",
    ];
    let mut result = HashMap::new();
    loop {
        let linesplit : Vec<&str> = trace_content.splitn(2, "\n").collect();
        if linesplit.len() != 2 {
            return Err(format!("Bad log({} parts) at '{}'", linesplit.len(), &trace_content[ .. 10]));
        }
        let line = linesplit[0];

        let parts :Vec<&str> = line.splitn(2, "=").collect();
        let (field, value) = (parts[0], parts[1]);
        if field == "a" {
            break;
        }
        trace_content = linesplit[1];
        if ALLOWED_KEYS.contains(&field) {
            result.insert(field.to_string(), value.to_string());
        } else {
            return Err(format!("Unknown key in kv: '{field}'"));
        }
    }
    Ok((result, trace_content))
}

fn validate_input(data: &[u8], expected_checksum_b64: &str) -> Result<(), String> {
    let actual = pki_util::sha256_b64(data);
    let want = expected_checksum_b64;
    if actual == want {
        Ok(())
    } else {
        Err(format!(
                "Checksum mismatch. Had='{actual}', wanted='{want}'", ))
    }
}


// given a line with key=value entries, return a hashmap representing the line.
// Values may be either hex, prefixed with 0x, or are otherwise decimal.
fn parse_fields(input: &str) -> HashMap<String, u16> {
    let parts = input.trim().split(" ");
    let mut result = HashMap::new();
    for part in parts {
        let kv : Vec<&str> = part.split("=").collect();
        assert_eq!(2, kv.len(), "Log parse failure: '{input}' ");
        let key = kv[0];
        let (val, radix) = match kv[1].strip_prefix("0x") {
            Some(v) => (v, 16),
            None => (kv[1], 10),
        };
        let val = u16::from_str_radix(val, radix).unwrap();
        result.insert(key.to_string(), val); 
    }
    result
}

fn assert_field(name: &str, want: u16, have: u16, line: usize) {
    assert_eq!(
        want, have,
        "{name} mismatch on line {line}. Have={have:04X} Want={want:04X}");
}
// Run the model in a given environment, and ensure the model's trace
// matches the trace from the real chip.
// All the following must be met:
// 1. The signature must validate the trace
// 2. The input must match the one in the trace
// 3. The model bus signals match the trace after each cycle (starting
// from the first reset read)
fn run_trace_test(
    checker: &pki_util::trace::TraceChecker,
    log_path: &str, input_path: &str) -> Result<(), String> {
    let log_data = std::fs::read_to_string(log_path)
        .or(Err("Failed to read log file.".to_string()))?;
    let log_data = checker.verify_trace(&log_data)?;
    let (kv, log_data) = get_trace_kv(log_data)?;

    let input_data : Vec<u8> = std::fs::read(input_path)
        .or(Err(format!("Failed to read input file: '{input_path}'")))?;
    let want_checksum = kv.get("InputSha256")
        .ok_or("Input checksum missing from log.".to_string())?;
    validate_input(&input_data, want_checksum)?;

    assert_model_log(&log_data, &input_data)?;
    Ok(())
}

#[cfg(test)]
mod test_utils {
    use super::*;
    // Tests for the test framework.
    #[test]
    fn test_kv() {
        let (kv, rest) = get_trace_kv(r#"InputSha256=XA2eNCnK6MOju3JTVGgsMRSv/huAlp7IEqmPevSX874=
a=0x0002 rwb=1 
a=0xFFFF rwb=1"#).unwrap();
        assert_eq!("XA2eNCnK6MOju3JTVGgsMRSv/huAlp7IEqmPevSX874=", kv["InputSha256"]);
        assert_eq!(true, rest.starts_with("a=0x0002 rwb=1"), "Actual='{rest}'");
    }
}

// Assert that the model matches the log, for all cycles including
// the first reset vector reads.
fn assert_model_log(log: &str, environment: &[u8])
    -> Result<(), String> {
    let mut cpu = W6502::new();
    let mut log = log.lines();
    let skipped_lines = reset_model(&mut cpu, &mut log);

    for (num, line) in log.enumerate() {
        println!("log: {line}");
        let num = num + 6;
        let fields = parse_fields(&line);
        cpu.cycle(&Inputs {
            data: environment[cpu.outputs().address as usize],
            clk: false, /*unused*/
            n_reset: true,
        });

        // Every line should have a and rwb
        assert_field("addr", fields["a"], cpu.outputs().address, num);
        assert_field("rwb", fields["rwb"], cpu.outputs().rwb as u16, num);


        // d(ata) is optional
        // check_field_option("data", fields["d"], cpu.outputs().d, fields["d"]);

    }
    Ok(())
}

// Reset the cpu, and step until the chip should
// be reading the reset vector.
// Returns the number of skipped log lines.
fn reset_model(cpu: &mut W6502, lines: &mut std::str::Lines) -> usize {
    let mut inputs = Inputs {
        clk: false,
        data: 0xca,
        n_reset: false,
    };
    for i in 0 .. 2 {
        cpu.cycle(&inputs);
    }
    inputs.n_reset = true;
    const SKIPPED_LINES : usize = 6;
    for i in 0 .. SKIPPED_LINES {
        cpu.cycle(&inputs);
        lines.next();
    }
    SKIPPED_LINES
}

#[cfg(test)]
mod trace_tests {
    use super::*;
    use pki_util::trace::TraceChecker;

    #[test]
    fn test_nop_jmp() {
        let checker = TraceChecker::new(
            &std::fs::read("chiplab_trace_signing.bin.pub").unwrap());
        // this test just nops in a loop.
        assert_eq!(
            Ok(()),
            run_trace_test(&checker, "passing_traces/nop_jmp_loop.log", "passing_traces/nop_jmp_loop.bin"));

    }

}
