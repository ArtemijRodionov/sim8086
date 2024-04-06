use std::collections::HashSet;

#[derive(Debug, Default)]
struct CmdOptions {
    flags: HashSet<String>,
    exec_path: String,
    dump_path: String,
}

fn help() {
    println!(
        r#"
Decode and emulate 8086 assembly

Usage: sim8086 [decode/emulate] [flags] [compiled 8086 assembly file]

Flags:
--help  prints help

Commands:
* `decode` - decodes 8086 assembly instructions
* `emulate` - emulates 8086 assembly
    * Flags
        * `--quite` disables printing
        * `--print-ip` prints ip changes 
        * `--print-estimates` prints clock's cycles estimation for instructions
        * `--dump-memory [name]` creates a file with name [name] and dumps emulator's memory into it
"#
    );
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args().skip(1);
    if args.len() == 0 {
        help();
    }

    let command = args.next().unwrap();
    let options = args.fold(CmdOptions::default(), |mut args, s| {
        if args.flags.contains("dump") && args.dump_path.is_empty() {
            if s.starts_with("--") {
                help();
            }
            args.dump_path = s.to_string();
        } else if s.starts_with("--") {
            args.flags.insert(s.trim_start_matches("--").to_string());
        } else if args.exec_path.is_empty() {
            args.exec_path = s.to_string();
        } else {
            help();
        }
        args
    });

    if options.exec_path.is_empty() || options.flags.contains("help") {
        help();
    }

    if command == "emulate" {
        let data = std::fs::read(&options.exec_path).expect("Can't open given file");
        let decoded = sim8086::decoder::decode(data.into_iter());
        let code: sim8086::emulator::Code = decoded
            .into_iter()
            .map(|x| x.expect("can't decode it"))
            .collect();

        let mut emulator = sim8086::emulator::Emulator::new(code);
        let mut tracer =
            sim8086::emulator::Tracer::with_options(sim8086::emulator::TracerOptions {
                with_ip: options.flags.contains("print-ip"),
                with_estimate: options.flags.contains("print-estimates"),
                with_trace: !options.flags.contains("quite"),
                dump_path: options.dump_path,
            });
        tracer.run(&mut emulator);
    } else if command == "decode" {
        let data = std::fs::read(&options.exec_path).expect("Can't open given file");
        let decoded = sim8086::decoder::decode(data.into_iter());
        for inst in decoded {
            match inst.map(|x| x.decode()) {
                Ok(op) => println!("{}", op),
                Err(e) => println!("{}", e),
            };
        }
    } else {
        help();
    }
}
