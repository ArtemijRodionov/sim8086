import sys
import glob
import difflib
import subprocess

default_decode_src = [37, 38, 39, 40, 41]
default_exec_src = [43, 44, 46]


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


def lines(xs):
    return [x.rstrip() for x in xs.splitlines()]


def diff(a, b):
    return "\n".join(
        color_diff(line)
        for line in difflib.unified_diff(lines(a), lines(b))
    )


def read(path):
    with open(path) as f:
        return f.read().strip()


def clean_src(xs):
    return "\n".join([
        x for x in xs.splitlines()
        if x
            and not x.startswith(';')
            and not x.startswith("bits")
            and not x.startswith("---")
    ])


def run_decode(obj_path, exec):
    args = ["./sim8086.bin", obj_path]
    if exec:
        args.append("--exec")
    result = subprocess.run(args, capture_output=True)
    if result.returncode != 0:
        raise Exception(result.stderr.decode().strip())
    return result.stdout.decode().strip()


def test(src, obj, exec):
    s = clean_src(read(src))
    o = run_decode(obj, exec)
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


def test_decode(number):
    glober = glob_it(number)
    asm = get_asm(glober)
    obj = get_obj(glober)
    test(asm, obj, False)


def test_exec(number):
    glober = glob_it(number)
    asm = get_txt(glober)
    obj = get_obj(glober)
    test(asm, obj, True)


def main():
    number_to_test = None
    if len(sys.argv) == 2:
        number_to_test = int(sys.argv[1])

    if number_to_test:
        if number_to_test in default_decode_src:
            test_decode(number_to_test)
        elif number_to_test in default_exec_src:
            test_exec(number_to_test)
        else:
            raise ValueError("Don't know such a test", number_to_test)
        return

    for s in default_decode_src:
        test_decode(s)

    for s in default_exec_src:
        test_exec(s)

if __name__ == "__main__":
    main()

