package main

import (
	"bufio"
	"fmt"
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

func gen() string {
	return fmt.Sprintf("{\"x0\": %f, \"y0\": %f,\"x1\": %f,\"y1\": %f}", randCoord(), randCoord(), randCoord(), randCoord())
}

func genRandLine(count int, ch chan string, wg *sync.WaitGroup) {
	for i := 0; i < count; i++ {
		ch <- ("," + gen())
	}
	wg.Done()
}

func writeRandLine(w *bufio.Writer, ch chan string) {
	for s := range ch {
		if _, err := w.WriteString(s); err != nil {
			panic(err)
		}
	}
}

func scheduleGen(split int, ch chan string) {
	line_num := 10_000_000 / split
	var wg sync.WaitGroup
	wg.Add(split)
	for i := 0; i < split; i++ {
		go genRandLine(line_num, ch, &wg)
	}
	wg.Wait()
	close(ch)
}

func main() {
	file, err := os.Create("data_10000000_flex.json")
	if err != nil {
		panic(err)
	}
	defer file.Close()
	writer := bufio.NewWriter(file)
	defer writer.Flush()
	writer.WriteString("{\"pairs\":[")
	ch := make(chan string, 1_000_000)
	ch <- gen()
	go scheduleGen(2, ch)
	writeRandLine(writer, ch)
	writer.WriteString("]}")
}
