package inst

import (
	"fmt"
)

func parseRegEncoding(inst inst) string {
	lhs := regEncoding{inst.reg, inst.size}
	rhs := regEncoding{inst.rm, inst.size}
	return fmt.Sprintf("%s %s, %s", inst.name, lhs, rhs)
}

type regEncoding struct {
	register
	opSize
}

func (r regEncoding) String() string {
	val, ok := regString[r]
	if !ok {
		panic("Unreachable")
	}
	return val
}

var regString = map[regEncoding]string{
	{alax, opByte}: "al",
	{clcx, opByte}: "cl",
	{dldx, opByte}: "dl",
	{blbx, opByte}: "bl",
	{ahsp, opByte}: "ah",
	{chbp, opByte}: "ch",
	{dhsi, opByte}: "dh",
	{bhdi, opByte}: "bh",
	{alax, opWord}: "ax",
	{clcx, opWord}: "cx",
	{dldx, opWord}: "dx",
	{blbx, opWord}: "bx",
	{ahsp, opWord}: "sp",
	{chbp, opWord}: "bp",
	{dhsi, opWord}: "si",
	{bhdi, opWord}: "di",
}
