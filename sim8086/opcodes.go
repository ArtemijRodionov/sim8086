package sim

type First byte
type D byte
type W byte

type Second byte
type Mod byte
type Reg byte
type RM byte

type Instruction struct {
	First
	Second
}

const (
	byteOperation W = 0x0
	wordOperation W = 0x1
)

const (
	MemoryNoDisplacement   Mod = 0x0 << 6
	MemoryDisplacement8    Mod = 0x1 << 6
	MemoryDisplacement16   Mod = 0x2 << 6
	RegisterNoDisplacement Mod = 0x3 << 6
)

func (o First) W() W {
	switch new := W(o & 1); new {
	case byteOperation, wordOperation:
		return new
	default:
		panic("can't read D")
	}
}
func (o First) D() D {
	return D(o & (1 << 1))
}
func (o Second) Mod() Mod {
	return Mod(0)
}
func (o Second) Reg() Reg {
	return Reg(0)
}
func (o Second) RM() RM {
	return RM(0)
}

type regEncoding struct {
	Reg
	W
}

var registerEncoding = map[regEncoding]string{
	{0x0 << 3, byteOperation}: "al",
	{0x1 << 3, byteOperation}: "cl",
	{0x2 << 3, byteOperation}: "dl",
	{0x3 << 3, byteOperation}: "bl",
	{0x4 << 3, byteOperation}: "ah",
	{0x5 << 3, byteOperation}: "ch",
	{0x6 << 3, byteOperation}: "dh",
	{0x7 << 3, byteOperation}: "bh",
	{0x0 << 3, wordOperation}: "ax",
	{0x1 << 3, wordOperation}: "cx",
	{0x2 << 3, wordOperation}: "dx",
	{0x3 << 3, wordOperation}: "bx",
	{0x4 << 3, wordOperation}: "sp",
	{0x5 << 3, wordOperation}: "bp",
	{0x6 << 3, wordOperation}: "si",
	{0x7 << 3, wordOperation}: "di",
}

func (r regEncoding) String() string {
	val, ok := registerEncoding[r]
	if !ok {
		panic("Can't encode register")
	}
	return val
}
