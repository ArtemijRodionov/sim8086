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

func (m ModeOffset) Validate() error {
	switch m {
	case regOffset0, memOffset0, memOffset8, memOffset16:
		return nil
	default:
		return errors.New(fmt.Sprintf("Can't parse mode %x", m))
	}
}

type InstEncoding interface {
	Name() string
	Mode() ModeOffset
	Reg() Register
	RM() Register
}

type InstRegEncoding interface {
	InstEncoding

	W() OpSize
}

type InstDirectAddrEncoding interface {
	InstEncoding

	DirectAddress() string
}

type InstDisplacementAddrEncoding interface {
	InstEncoding

	Displacement(bool) int16
}

type Inst struct {
	InstEncoding
}

func New(i InstEncoding) Inst {
	return Inst{i}
}

func (i Inst) Parse() (string, error) {
	mode := i.Mode()
	if err := mode.Validate(); err != nil {
		return "", err
	}

	var result = ""
	if mode == regOffset0 {
		inst, ok := i.InstEncoding.(InstRegEncoding)
		if !ok {
			return "", errors.New(fmt.Sprintf(
				"Instraction %s doesn't implement required interface InstRegEncoding", i.Name()))
		}
		result = parseRegEncoding(inst)
	} else {

	}
	return result, nil
}
