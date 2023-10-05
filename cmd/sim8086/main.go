package main

import (
	"bufio"
	"flag"
	"fmt"
	"log"
	"os"
	"strings"

	"github.com/artemijrodionov/performance-aware-programming/sim8086"
)

var objPath = flag.String("objPath", "", "Unix path to a binary file compiled with nasm")

func scanTwoBytes(data []byte, atEOF bool) (advance int, token []byte, err error) {
	if atEOF && len(data) == 0 {
		return 0, nil, nil
	}

	return len(data[0:2]), data[0:2], nil
}

func isObjFile(filename string) bool {
	return filename != "" && !strings.HasSuffix(filename, ".asm")
}

func main() {
	flag.Parse()
	if !isObjFile(*objPath) {
		flag.Usage()
		os.Exit(1)
	}
	file, err := os.Open(*objPath)
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()

	scanner := bufio.NewScanner(bufio.NewReader(file))
	scanner.Split(scanTwoBytes)
	for scanner.Scan() {
		bytes := scanner.Bytes()
		inst := sim8086.NewInstruction(bytes[0], bytes[1])
		fmt.Println(inst)
	}

	if err := scanner.Err(); err != nil {
		log.Fatal(err)
	}
}
