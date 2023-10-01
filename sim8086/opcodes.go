package sim8086

import "fmt"

type First byte
type Opcode byte
type D byte
type W byte

const (
	directionSrc D = 0x0
	directionDst D = 0x1
)

const (
	operationByte W = 0x0
	operationWord W = 0x1
)

func (o First) W() W {
	switch w := W(o & 1); w {
	case operationByte, operationWord:
		return w
	default:
		panic("can't read W")
	}
}

func (o First) D() D {
	switch d := D(o & (1 << 1)); d {
	case directionSrc, directionDst:
		return d
	default:
		panic("can't parse D")
	}
}

type Second byte
type Mod byte
type Reg byte

const (
	modMemOffset0  Mod = 0x0
	modMemOffset8  Mod = 0x1
	modMemOffset16 Mod = 0x2
	modRegOffset0  Mod = 0x3
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

func (o Second) Mod() Mod {
	switch d := Mod((o & 0xe0) >> 6); d {
	case modMemOffset0, modMemOffset8, modMemOffset16, modRegOffset0:
		return d
	default:
		panic("can't parse Mod")
	}
}

func (o Second) Reg() Reg {
	switch r := Reg((o & 0x38) >> 3); r {
	case alax, clcx, dldx, blbx, ahsp, chbp, dhsi, bhdi:
		return r
	default:
		panic("can't parse Reg")
	}
}

func (o Second) RM() Reg {
	switch r := Reg(o & 0x7); r {
	case alax, clcx, dldx, blbx, ahsp, chbp, dhsi, bhdi:
		return r
	default:
		panic("can't parse Reg")
	}
}

type regEncoding struct {
	Reg
	W
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
	First
	Second
}

func (i Instruction) String() string {
	lhs := regEncoding{i.RM(), i.W()}
	rhs := regEncoding{i.Reg(), i.W()}

	return fmt.Sprintf("mov %s, %s", lhs, rhs)
}

func NewIntruction(first, second byte) Instruction {
	return Instruction{First(first), Second(second)}
}
