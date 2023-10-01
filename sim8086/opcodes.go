package sim8086

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
	modMemOffset0  Mod = 0x0 << 6
	modMemOffset8  Mod = 0x1 << 6
	modMemOffset16 Mod = 0x2 << 6
	modRegOffset0  Mod = 0x3 << 6
)

const (
	alax = 0x0 << 3
	clcx = 0x1 << 3
	dldx = 0x2 << 3
	blbx = 0x3 << 3
	ahsp = 0x4 << 3
	chbp = 0x5 << 3
	dhsi = 0x6 << 3
	bhdi = 0x7 << 3
)

func (o Second) Mod() Mod {
	switch d := Mod(o & 0xe0); d {
	case modMemOffset0, modMemOffset8, modMemOffset16, modRegOffset0:
		return d
	default:
		panic("can't parse Mod")
	}
}

func (o Second) Reg() Reg {
	return Reg(0)
}

func (o Second) RM() Reg {
	return Reg(0)
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

func ParseIntruction(first, second byte) Instruction {
	return Instruction{}
}
