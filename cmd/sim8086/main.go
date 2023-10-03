package main

import "flag"
import "bufio"
import "os"
import "log"
import "fmt"

import "github.com/artemijrodionov/performance-aware-programming/sim8086"

var objPath = flag.String("objPath", "", "Unix path to an ASM obj file")

func ScanTwoBytes(data []byte, atEOF bool) (advance int, token []byte, err error) {
	if atEOF && len(data) == 0 {
		return 0, nil, nil
	}

	return len(data[0:2]), data[0:2], nil
}

func main() {
	flag.Parse()
	if *objPath == "" {
		flag.Usage()
		os.Exit(1)
	}
	file, err := os.Open(*objPath)
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()

	scanner := bufio.NewScanner(bufio.NewReader(file))
	scanner.Split(ScanTwoBytes)
	for scanner.Scan() {
		bytes := scanner.Bytes()
		inst := sim8086.NewInstruction(bytes[0], bytes[1])
		fmt.Println(inst)
	}

	if err := scanner.Err(); err != nil {
		log.Fatal(err)
	}
}
