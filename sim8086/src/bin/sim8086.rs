use std::{collections::HashSet, env::args};

#[derive(Debug, Default)]
struct CmdOptions {
    flags: HashSet<String>,
    asm_path: String,
    dump_path: String,
}

fn main() {
    let options = args()
        .into_iter()
        .skip(1)
        .try_fold(CmdOptions::default(), |mut args, s| {
            if s.starts_with("--") {
                args.flags.insert(s.trim_start_matches("--").to_string());
            } else if args.flags.contains("dump") && args.dump_path == "" {
                args.dump_path = s.to_string();
            } else if args.asm_path == "" {
                args.asm_path = s.to_string();
            } else {
                return Err("You can't have multiple paths");
            }

            Ok(args)
        })
        .expect("Provide unix path to 8086 binary file");

    let data = std::fs::read(&options.asm_path).expect("Can't open given file");
    let asm_ops = sim8086::decoder::parse(data.into_iter());

    if options.flags.is_empty() || options.flags.contains("help") {
        println!(
            r#"
Decoder and interpreter for 8086 assembler

Usage: sim8086 [--flags] [compiled assembly file]
Flags:
    --help  prints help
    --print prints human-readable result
        --ip prints ip changes during printing

    --exec  interpretes a decoded assembly file. Without this flag will only decode an assembly file.
        --dump [file] offload memory after execution to a given file
"#
        )
    } else if options.flags.contains("exec") {
        let asm_ops: sim8086::interpreter::Code = asm_ops
            .into_iter()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        let mut processor = sim8086::interpreter::Processor::new(asm_ops);

        let mut tracer =
            sim8086::interpreter::Tracer::with_options(sim8086::interpreter::TracerOptions {
                with_ip: options.flags.contains("ip"),
                with_print: options.flags.contains("print"),
                dump_path: options.dump_path,
                ..sim8086::interpreter::TracerOptions::default()
            });

        tracer.run(&mut processor);
    } else {
        for inst in asm_ops {
            match inst.and_then(|x| Ok(x.decode())) {
                Ok(op) => println!("{}", op.to_string()),
                Err(e) => println!("{}", e),
            };
        }
    }
}
