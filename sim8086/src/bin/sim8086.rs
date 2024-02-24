use std::{collections::HashSet, env::args};

#[derive(Debug, Default)]
struct CmdOptions {
    flags: HashSet<String>,
    path: String,
}

fn main() {
    let options = args()
        .into_iter()
        .skip(1)
        .try_fold(CmdOptions::default(), |mut args, s| {
            if s.starts_with("--") {
                args.flags.insert(s.trim_start_matches("--").to_string());
            } else if args.path == "" {
                args.path = s.to_string();
            } else {
                return Err("You can't have multiple paths");
            }

            Ok(args)
        })
        .expect("Provide unix path to 8086 binary file");

    let data = std::fs::read(&options.path).expect("Can't open given file");
    let asm_ops = sim8086::decoder::parse(data.into_iter());

    if options.flags.is_empty() {
        for inst in asm_ops {
            match inst.and_then(|x| Ok(x.decode())) {
                Ok(op) => println!("{}", op.to_string()),
                Err(e) => println!("{}", e),
            };
        }
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
                ..sim8086::interpreter::TracerOptions::default()
            });
        tracer.run(&mut processor);
    } else {
        panic!("Unknown options {:?}", options);
    }
}
