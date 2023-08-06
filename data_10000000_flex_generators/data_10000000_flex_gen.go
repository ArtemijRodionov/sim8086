package main

import (
	"bufio"
	"fmt"
	"math/rand"
	"os"
)

func randCoord() float32 {
	degree := rand.Float32()*180 + rand.Float32()*999999/1000000
	if rand.Intn(2) == 0 {
		return degree
	}
	return -degree
}

func writeRandLine(w *bufio.Writer) {
	if _, err := fmt.Fprintf(w, "{\"x0\": %f, \"y0\": %f,\"x1\": %f,\"y1\": %f}", randCoord(), randCoord(), randCoord(), randCoord()); err != nil {
		panic(err)
	}
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
	writeRandLine(writer)
	for i := 0; i < 10_000_000; i++ {
		writer.WriteByte(',')
		writeRandLine(writer)
	}
	writer.WriteString("]}")
}
