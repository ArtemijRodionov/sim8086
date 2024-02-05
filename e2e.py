import glob
import difflib
import subprocess

default_decode_src = [37, 38, 39, 40, 41]
default_exec_src = [43]


def read(path):
    with open(path) as f:
        return f.read()


def clean_src(xs):
    return "\n".join([
        x for x in xs.splitlines()
        if x
            and not x.startswith(';')
            and not x.startswith("bits")
            and not x.startswith("---")
    ])


def color_diff(line):
    if line.startswith("-"):
        return red(line)
    if line.startswith('+'):
        return green(line)
    return line


def diff(a, b):
    return "\n".join(
        color_diff(line)
        for line in difflib.unified_diff(a.splitlines(), b.splitlines())
    )


def run_decode(obj_path, exec):
    args = ["./sim8086.bin", obj_path]
    if exec:
        args.append("--exec")
    result = subprocess.run(args, capture_output=True)
    if result.returncode != 0:
        raise Exception(result.stderr.decode())
    return result.stdout.decode()


def green(msg):
    return f"\033[1;32m{msg}\033[0m"


def red(msg):
    return f"\033[1;31m{msg}\033[0m"


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


def main():
    for s in default_decode_src:
        glober = glob_it(s)
        asm = get_asm(glober)
        obj = get_obj(glober)
        test(asm, obj, False)

    for s in default_exec_src:
        glober = glob_it(s)
        asm = get_txt(glober)
        obj = get_obj(glober)
        test(asm, obj, True)


if __name__ == "__main__":
    main()

