package inst

import (
	"errors"
	"fmt"
)

type Register byte
type OpSize byte
type ModeOffset byte

const (
	opByte OpSize = 0x0
	opWord OpSize = 0x1

	memOffset0  ModeOffset = 0x0
	memOffset8  ModeOffset = 0x1
	memOffset16 ModeOffset = 0x2
	regOffset0  ModeOffset = 0x3

	alax Register = 0x0
	clcx Register = 0x1
	dldx Register = 0x2
	blbx Register = 0x3
	ahsp Register = 0x4
	chbp Register = 0x5
	dhsi Register = 0x6
	bhdi Register = 0x7
)

type InstEncoding interface {
	Mode() ModeOffset
	Name() string
}

type InstRegEncoding interface {
	InstEncoding

	W() OpSize
	Reg() Register
	RM() Register
}

type InstAddrEncoding interface {
	InstEncoding

	Reg() Register
	Disp() string
}

type Inst struct {
	InstEncoding
}

func New(i InstEncoding) Inst {
	return Inst{i}
}

func (i Inst) Parse() (string, error) {
	var result = ""
	switch i.Mode() {
	case memOffset0:
		inst, ok := i.InstEncoding.(InstRegEncoding)
		if !ok {
			return "", errors.New(fmt.Sprintf(
				"Instraction %s doesn't implement required interface InstRegEncoding", i.Name()))
		}
		result = parseRegEncoding(inst)
	}
	return result, nil
}
