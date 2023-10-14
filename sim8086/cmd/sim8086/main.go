package main

import (
	"bufio"
	"errors"
	"flag"
	"fmt"
	"log"
	"os"
	"strings"

	"github.com/artemijrodionov/sim8086/inst"
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

type Cli struct {
	ObjPath string
}

func NewCli(objPath string) (*Cli, error) {
	if !isObjFile(objPath) {
		return nil, errors.New("Executable file name is not valid")
	}
	return &Cli{ObjPath: objPath}, nil
}

func (c *Cli) Run() error {
	file, err := os.Open(c.ObjPath)
	if err != nil {
		return err
	}
	defer file.Close()

	scanner := bufio.NewScanner(bufio.NewReader(file))
	scanner.Split(scanTwoBytes)
	for scanner.Scan() {
		bytes := scanner.Bytes()
		inst := inst.NewInstruction(bytes[0], bytes[1])
		fmt.Println(inst)
	}

	if err := scanner.Err(); err != nil {
		return err
	}

	return nil
}

func main() {
	flag.Parse()

	cli, err := NewCli(*objPath)
	if err != nil {
		flag.Usage()
		log.Fatal(err)
	}

	if err := cli.Run(); err != nil {
		log.Fatal(err)
	}
}
