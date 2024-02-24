use std::{
    collections::{HashMap, HashSet},
    env::args,
};

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
        let asm_ops: Vec<sim8086::decoder::Asm> = asm_ops
            .into_iter()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();
        let ip_idx: HashMap<usize, usize> = asm_ops
            .iter()
            .enumerate()
            .map(|iasm| (iasm.1.ip, iasm.0))
            .collect();

        let mut m = sim8086::interpreter::Machine::default();
        let mut tracer =
            sim8086::interpreter::Tracer::with_options(sim8086::interpreter::TracerOptions {
                with_ip: options.flags.contains("ip"),
                ..sim8086::interpreter::TracerOptions::default()
            });

        let mut ip = 0;
        let last_ip = asm_ops.last().unwrap().ip;
        while ip <= last_ip {
            let inst = asm_ops[ip_idx[&ip]].decode();
            ip = tracer.trace_exec(&mut m, inst) as usize;
        }
        tracer.trace_state(&m);
    } else {
        panic!("Unknown options {:?}", options);
    }
}
