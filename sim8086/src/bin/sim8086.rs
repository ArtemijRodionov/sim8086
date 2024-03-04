use std::{collections::HashSet, env::args};

#[derive(Debug, Default)]
struct CmdOptions {
    flags: HashSet<String>,
    asm_path: String,
    dump_path: String,
}

fn main() {
    let options = args()
        .skip(1)
        .try_fold(CmdOptions::default(), |mut args, s| {
            if s.starts_with("--") {
                args.flags.insert(s.trim_start_matches("--").to_string());
            } else if args.flags.contains("dump") && args.dump_path.is_empty() {
                args.dump_path = s.to_string();
            } else if args.asm_path.is_empty() {
                args.asm_path = s.to_string();
            } else {
                return Err("You can't have multiple paths");
            }

            Ok(args)
        })
        .expect("Provide unix path to 8086 binary file");

    if options.flags.is_empty() || options.flags.contains("help") {
        println!(
            r#"
Decoder and interpreter for 8086 assembler

Usage: sim8086 [--flags] [compiled assembly file]
Flags:
    --help  prints help
    --print prints human-readable result. Without this flag there won't be console prints
    --exec  interpretes a decoded assembly file. Without this flag will only decode an assembly file.
        --ip prints ip changes during printing 
        --estimate prints opcodes estimate during printing
        --dump [file] offload memory after execution to a given file
"#
        )
    } else if options.flags.contains("exec") {
        let data = std::fs::read(&options.asm_path).expect("Can't open given file");
        let asm_ops = sim8086::decoder::parse(data.into_iter());
        let asm_ops: sim8086::interpreter::Code = asm_ops
            .into_iter()
            .filter_map(|x| x.ok())
            .collect();

        let mut processor = sim8086::interpreter::Processor::new(asm_ops);

        let mut tracer =
            sim8086::interpreter::Tracer::with_options(sim8086::interpreter::TracerOptions {
                with_ip: options.flags.contains("ip"),
                with_print: options.flags.contains("print"),
                with_estimate: options.flags.contains("estimate"),
                dump_path: options.dump_path,
                ..sim8086::interpreter::TracerOptions::default()
            });

        tracer.run(&mut processor);
    } else {
        let data = std::fs::read(&options.asm_path).expect("Can't open given file");
        let asm_ops = sim8086::decoder::parse(data.into_iter());
        for inst in asm_ops {
            match inst.map(|x| x.decode()) {
                Ok(op) => println!("{}", op),
                Err(e) => println!("{}", e),
            };
        }
    }
}
