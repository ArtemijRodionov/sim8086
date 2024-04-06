from dataclasses import dataclass
import sys
import glob
import difflib
import subprocess


def green(msg):
    return f"\033[1;32m{msg}\033[0m"


def red(msg):
    return f"\033[1;31m{msg}\033[0m"


def color_diff(line):
    if line.startswith("-"):
        return red(line)
    if line.startswith('+'):
        return green(line)
    return line


def diff_lines(xs):
    return [x.strip() for x in xs.splitlines()]


def diff(a, b):
    return "\n".join(
        color_diff(line)
        for line in difflib.unified_diff(diff_lines(a), diff_lines(b))
    )


def read(path):
    with open(path) as f:
        while (line := f.readline()):
            yield line.strip()


def filter_lines(xs):
    for x in xs:
        keep = (x.strip()
                and not x.strip().startswith(';')
                and not x.startswith("bits")
                and not x.startswith("---"))
        if keep:
            yield x


def join(xs):
    return "\n".join(xs)


def remove_comments(xs):
    for x in xs:
        yield x.split(';')[0].strip()


def read_asm(src, keep_comments):
    s = filter_lines(read(src))
    if not keep_comments:
        s = remove_comments(s)
    return join(s)


class Command:
    Decode = "decode"
    Emulate = "emulate"


@dataclass
class TestOptions:
    command: str = Command.Decode
    print: bool = True
    dump: str = ""
    ip: bool = False
    keep_comments: bool = False
    estimate: bool = False


def run_decode(obj_path, options: TestOptions):
    args = ["./sim8086.bin", options.command]

    if options.ip:
        args.append("--print-ip")
    if options.estimate:
        args.append("--print-estimates")
    if options.dump:
        args.extend(["--dump-memory", options.dump])

    args.append(obj_path)
    result = subprocess.run(args, capture_output=True)
    if result.returncode != 0:
        raise Exception(result.stderr.decode().strip())
    return result.stdout.decode().strip()


def test(src, obj, options):
    s = read_asm(src, options.keep_comments)
    o = run_decode(obj, options)
    d = diff(s, o)
    if d:
        print(f"{red('Fail')}: {src}\n\n{d}")
        exit(1)
    else:
        print(f"{green('Success')}: {src}")


def glob_it(number):
    return glob.glob(f"data/*{number}*")


def get_ext(glober, ext):
    for g in glober:
        if g.endswith(ext):
            return g


def get_txt(glober):
    return get_ext(glober, ".txt")


def get_asm(glober):
    return get_ext(glober, ".asm")


def get_obj(glober):
    for g in glober:
        if '.' not in g:
            return g


def test_decode(number, options):
    glober = glob_it(number)
    asm = get_asm(glober)
    obj = get_obj(glober)
    test(asm, obj, options)


def test_emulate(number, options):
    glober = glob_it(number)
    asm = get_txt(glober)
    obj = get_obj(glober)
    test(asm, obj, options)


def main():
    decode_src = [37, 38, 39, 40, 41]
    emulate_src = [43, 44, 46]
    emulate_ip_src = [48, 49, 51, 52, 53, 54, 55]
    emulate_estimate_src = [56, 57]

    decode_opt = TestOptions()
    emulate_opt = TestOptions(command=Command.Emulate, keep_comments=True)
    emulate_ip_opt = TestOptions(
        command=Command.Emulate,
        ip=True,
        keep_comments=True,
    )
    emulate_est_opt = TestOptions(
        command=Command.Emulate,
        ip=True,
        estimate=True,
        keep_comments=True,
    )

    number_to_test = None
    if len(sys.argv) == 2:
        number_to_test = int(sys.argv[1])

    if number_to_test:
        if number_to_test in decode_src:
            test_decode(number_to_test, decode_opt)
        elif number_to_test in emulate_src:
            test_emulate(number_to_test, emulate_opt)
        elif number_to_test in emulate_ip_src:
            test_emulate(number_to_test, emulate_ip_opt)
        elif number_to_test in emulate_estimate_src:
            test_emulate(number_to_test, emulate_est_opt)
        else:
            raise ValueError("Don't know such a test", number_to_test)
        return

    for s in decode_src:
        test_decode(s, decode_opt)

    for s in emulate_src:
        test_decode(s, decode_opt)
        test_emulate(s, emulate_opt)

    for s in emulate_ip_src:
        test_decode(s, decode_opt)
        test_emulate(s, emulate_ip_opt)

    for s in emulate_estimate_src:
        test_decode(s, decode_opt)
        test_emulate(s, emulate_est_opt)


if __name__ == "__main__":
    main()
