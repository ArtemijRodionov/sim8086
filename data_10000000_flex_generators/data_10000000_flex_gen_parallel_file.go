package main

import (
	"bufio"
	"fmt"
	"io"
	"math/rand"
	"os"
	"sync"
)

func randCoord() float32 {
	degree := rand.Float32()*180 + rand.Float32()*999999/1000000
	if rand.Intn(2) == 0 {
		return degree
	}
	return -degree
}

func writeRandLine(w io.Writer) {
	if _, err := fmt.Fprintf(w, "{\"x0\": %f, \"y0\": %f,\"x1\": %f,\"y1\": %f}", randCoord(), randCoord(), randCoord(), randCoord()); err != nil {
		panic(err)
	}
}

func create(count int) string {
	file, err := os.CreateTemp(os.TempDir(), "data_10000000_flex")
	if err != nil {
		panic(err)
	}
	defer file.Close()
	writer := bufio.NewWriter(file)
	defer writer.Flush()
	for i := 0; i < count-1; i++ {
		writer.WriteByte(',')
		writeRandLine(writer)
	}
	return file.Name()
}

func merge(data []string) {
	file, err := os.Create("data_10000000_flex.json")
	if err != nil {
		panic(err)
	}
	defer file.Close()
	writer := bufio.NewWriter(file)
	defer writer.Flush()
	writer.WriteString("{\"pairs\":[")
	writeRandLine(writer)
	for _, file_name := range data {
		rfile, err := os.Open(file_name)
		reader := bufio.NewReader(rfile)
		if err != nil {
			panic(err)
		}
		io.Copy(writer, reader)
	}
	writer.WriteString("]}")
}

func schedule(chunks int) []string {
	lines_per_chunk := 10_000_000 / chunks
	println("chunks ", chunks)
	println("lines_per_chunk ", lines_per_chunk)
	var wg sync.WaitGroup

	result := make([]string, chunks)
	for i := 0; i < chunks; i++ {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			result[i] = create(lines_per_chunk)
		}(i)
	}

	wg.Wait()
	return result
}

func main() {
	merge(schedule(3))
}
