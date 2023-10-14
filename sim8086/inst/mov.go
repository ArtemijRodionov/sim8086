package inst

import "fmt"

/*
Probably I should create an interface to extract byte positions for D, W, Reg and R/W for other opcodes
I should think about different sizes of opcodes
*/

type byte1 byte
type opcode byte
type direction byte
type operationType byte

const (
	directionSrc direction = 0x0
	directionDst direction = 0x1
)

const (
	operationByte operationType = 0x0
	operationWord operationType = 0x1
)

func (o byte1) W() operationType {
	switch w := operationType(o & 1); w {
	case operationByte, operationWord:
		return w
	default:
		panic("can't read operationType")
	}
}

func (o byte1) D() direction {
	switch d := direction(o & (1 << 1)); d {
	case directionSrc, directionDst:
		return d
	default:
		panic("can't parse direction")
	}
}

type byte2 byte
type mode byte
type register byte

const (
	modMemOffset0  mode = 0x0
	modMemOffset8  mode = 0x1
	modMemOffset16 mode = 0x2
	modRegOffset0  mode = 0x3
)

const (
	alax = 0x0
	clcx = 0x1
	dldx = 0x2
	blbx = 0x3
	ahsp = 0x4
	chbp = 0x5
	dhsi = 0x6
	bhdi = 0x7
)

func (o byte2) Mod() mode {
	switch d := mode((o & 0xe0) >> 6); d {
	case modMemOffset0, modMemOffset8, modMemOffset16, modRegOffset0:
		return d
	default:
		panic("can't parse mode")
	}
}

func (o byte2) Reg() register {
	switch r := register((o & 0x38) >> 3); r {
	case alax, clcx, dldx, blbx, ahsp, chbp, dhsi, bhdi:
		return r
	default:
		panic("can't parse register")
	}
}

func (o byte2) RM() register {
	switch r := register(o & 0x7); r {
	case alax, clcx, dldx, blbx, ahsp, chbp, dhsi, bhdi:
		return r
	default:
		panic("can't parse register")
	}
}

type regEncoding struct {
	register
	operationType
}

var registerEncoding = map[regEncoding]string{
	{alax, operationByte}: "al",
	{clcx, operationByte}: "cl",
	{dldx, operationByte}: "dl",
	{blbx, operationByte}: "bl",
	{ahsp, operationByte}: "ah",
	{chbp, operationByte}: "ch",
	{dhsi, operationByte}: "dh",
	{bhdi, operationByte}: "bh",
	{alax, operationWord}: "ax",
	{clcx, operationWord}: "cx",
	{dldx, operationWord}: "dx",
	{blbx, operationWord}: "bx",
	{ahsp, operationWord}: "sp",
	{chbp, operationWord}: "bp",
	{dhsi, operationWord}: "si",
	{bhdi, operationWord}: "di",
}

func (r regEncoding) String() string {
	val, ok := registerEncoding[r]
	if !ok {
		panic("Can't encode register")
	}
	return val
}

type Instruction struct {
	byte1
	byte2
}

func (i Instruction) String() string {
	lhs := regEncoding{i.RM(), i.W()}
	rhs := regEncoding{i.Reg(), i.W()}

	return fmt.Sprintf("mov %s, %s", lhs, rhs)
}

func NewInstruction(first, second byte) Instruction {
	return Instruction{byte1(first), byte2(second)}
}
