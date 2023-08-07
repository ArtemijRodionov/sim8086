package main

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"math/rand"
	"strings"
)

type rnd struct {
	x0, y0, x1, y1 float32
}

func randCoord() float32 {
	degree := rand.Float32()*180 + rand.Float32()*999999/1000000
	if rand.Intn(2) == 0 {
		return degree
	}
	return -degree
}

func writeLine(w io.Writer, coord rnd) {
	pattern := "{\"x0\": %f, \"y0\": %f,\"x1\": %f,\"y1\": %f}"
	if _, err := fmt.Fprintf(w, pattern, coord.x0, coord.y0, coord.x1, coord.y1); err != nil {
		panic(err)
	}
}

func randLine() rnd {
	return rnd{randCoord(), randCoord(), randCoord(), randCoord()}
}

func writeRandLine(w io.Writer) {
	writeLine(w, randLine())
}

type genType string
type genTypes []genType

var sequential genType = "sequential"
var parallelInMemory genType = "parallel_mem"
var parallelInFile genType = "parallel_file"
var concurrent genType = "concurrent"
var generators genTypes = genTypes{
	sequential, parallelInMemory, parallelInFile, concurrent,
}

func (gs genTypes) String() string {
	generators := []genType(gs)
	result := make([]string, len(generators))
	for i, g := range generators {
		result[i] = string(g)
	}
	return strings.Join(result, ", ")
}

func (g genType) String() string {
	return string(g)
}

func (g *genType) Set(s string) error {
	for _, t := range generators {
		if t.String() == s {
			*g = genType(s)
			return nil
		}
	}
	return errors.New("can't find generator")
}

var threads int
var generator genType

func init() {
	flag.IntVar(&threads, "threads", 3, "how many threads to use?")
	flag.Var(&generator, "generator", "how to generate data? "+generators.String())
	flag.Parse()
}

func main() {
	switch generator {
	case sequential:
		SequentialGen()
	case concurrent:
		ConcurrentGen(threads)
	case parallelInFile:
		ParallelInFileGen(threads)
	case parallelInMemory:
		ParallelInMemoryGen(threads)
	default:
		flag.Usage()
	}
}
