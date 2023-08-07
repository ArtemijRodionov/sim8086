package main

import (
	"bufio"
	"bytes"
	"os"
	"sync"
)

func create(count int) string {
	writer := bytes.NewBufferString("")
	for i := 0; i < count; i++ {
		writer.WriteByte(',')
		writeRandLine(writer)
	}
	return writer.String()
}

func merge(data chan string) {
	file, err := os.Create("data_10000000_flex.json")
	if err != nil {
		panic(err)
	}
	defer file.Close()
	writer := bufio.NewWriter(file)
	defer writer.Flush()
	writer.WriteString("{\"pairs\":[")
	writeRandLine(writer)
	for lines := range data {
		writer.WriteString(lines)
	}
	writer.WriteString("]}")
}

func schedule(chunks int) chan string {
	lines_per_chunk := 10_000_000 / chunks
	println("chunks ", chunks)
	println("lines_per_chunk ", lines_per_chunk)
	var wg sync.WaitGroup

	ch := make(chan string, chunks)
	for i := 0; i < chunks; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			ch <- create(lines_per_chunk)
		}()
	}

	go func() {
		wg.Wait()
		close(ch)
	}()

	return ch
}

func ParallelInMemoryGen(threads int) {
	merge(schedule(threads))
}
