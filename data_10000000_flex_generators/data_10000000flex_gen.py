# coding: utf-8
import time
import random as rnd


def r(): return rnd.randrange(-179, 179) + rnd.randrange(0, 999999)/1000000


def gen_line():
    return '{{"x0": {}, "y0": {}, "x1": {},"y1": {}}}'.format(r(), r(), r(), r())


def gen():
    s = time.time()
    with open("data_10000000_flex.json", "w", buffering=1) as f:
        f.write('{"pairs":[')
        f.write(gen_line())
        for _ in range(10_000_000):
            f.write(',')
            f.write(gen_line())
        f.write(']}')
        f.flush()
    print(format(time.time() - s, ','), 'secs')


if __name__ == "__main__":
    gen()
