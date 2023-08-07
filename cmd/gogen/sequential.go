package main

import (
	"bufio"
	"os"
)

func SequentialGen() {
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
