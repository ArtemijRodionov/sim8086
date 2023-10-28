package inst

import (
	"fmt"
)

func parseRegEncoding(inst InstRegEncoding) string {
	lhs := regEncoding{inst.Reg(), inst.W()}
	rhs := regEncoding{inst.RM(), inst.W()}
	return fmt.Sprintf("%s %s, %s", inst.Name(), lhs, rhs)
}

type regEncoding struct {
	Register
	OpSize
}

func (r regEncoding) String() string {
	val, ok := regString[r]
	if !ok {
		panic("Can't encode register")
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
