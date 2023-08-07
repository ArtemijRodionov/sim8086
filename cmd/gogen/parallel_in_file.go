package main

import (
	"bufio"
	"io"
	"os"
	"sync"
)

func create_file(count int) string {
	file, err := os.CreateTemp(os.TempDir(), "data_10000000_flex")
	if err != nil {
		panic(err)
	}
	defer file.Close()
	writer := bufio.NewWriter(file)
	defer writer.Flush()
	for i := 0; i < count; i++ {
		writer.WriteByte(',')
		writeRandLine(writer)
	}
	return file.Name()
}

func merge_files(data []string) {
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
		if err != nil {
			panic(err)
		}
		reader := bufio.NewReader(rfile)
		io.Copy(writer, reader)
	}
	writer.WriteString("]}")
}

func schedule_creation(chunks int) []string {
	lines_per_chunk := 10_000_000 / chunks
	println("chunks ", chunks)
	println("lines_per_chunk ", lines_per_chunk)
	result := make([]string, chunks)
	var wg sync.WaitGroup

	for i := 0; i < chunks; i++ {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			result[i] = create_file(lines_per_chunk)
		}(i)
	}

	wg.Wait()
	return result
}

func ParallelInFileGen(threads int) {
	merge_files(schedule_creation(threads))
}
