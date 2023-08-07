package main

import (
	"bufio"
	"os"
	"sync"
)

type rndCh chan rnd

func writeFromCh(w *bufio.Writer, ch rndCh) {
	for s := range ch {
		w.WriteByte(',')
		writeLine(w, s)
	}
}

func scheduleGen(chunks int, ch rndCh) {
	line_num := 10_000_000 / chunks
	println("chunks", chunks)
	println("line_per_chunk", line_num)
	var wg sync.WaitGroup
	wg.Add(chunks)
	for i := 0; i < chunks; i++ {
		go func() {
			for i := 0; i < line_num; i++ {
				ch <- randLine()
			}
			wg.Done()
		}()
	}
	wg.Wait()
	close(ch)
}

func ConcurrentGen(threads int) {
	file, err := os.Create("data_10000000_flex.json")
	if err != nil {
		panic(err)
	}
	defer file.Close()
	writer := bufio.NewWriter(file)
	defer writer.Flush()
	writer.WriteString("{\"pairs\":[")
	writeRandLine(writer)

	ch := make(rndCh, 10_000_000)
	go scheduleGen(threads, ch)
	writeFromCh(writer, ch)

	writer.WriteString("]}")
}
