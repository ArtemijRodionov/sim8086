package inst

import (
	"errors"
	"fmt"
)

type opDirection byte
type opSize byte
type modeOffset byte
type register byte

const (
	opSrc opDirection = 0x0
	opDst opDirection = 0x1

	opByte opSize = 0x0
	opWord opSize = 0x1

	memOffset0  modeOffset = 0x0
	memOffset8  modeOffset = 0x1
	memOffset16 modeOffset = 0x2
	regOffset0  modeOffset = 0x3

	alax register = 0x0
	clcx register = 0x1
	dldx register = 0x2
	blbx register = 0x3
	ahsp register = 0x4
	chbp register = 0x5
	dhsi register = 0x6
	bhdi register = 0x7
)

func (o opDirection) validate() error {
	switch o {
	case opDst, opSrc:
		return nil
	default:
		return errors.New(fmt.Sprintf("Can't parse op direction %x", o))
	}
}

func (o opSize) validate() error {
	switch o {
	case opByte, opWord:
		return nil
	default:
		return errors.New(fmt.Sprintf("Can't parse op size %x", o))
	}
}

func (m modeOffset) validate() error {
	switch m {
	case regOffset0, memOffset0, memOffset8, memOffset16:
		return nil
	default:
		return errors.New(fmt.Sprintf("Can't parse mode offset %x", m))
	}
}

func (r register) validate() error {
	switch r {
	case alax, clcx, dldx, blbx, ahsp, chbp, dhsi, bhdi:
		return nil
	default:
		return errors.New(fmt.Sprintf("Can't parse register %x", r))
	}
}

/*
1. Делаю структуру, которая с byte1 и byte2
2.1 Делаю callback для direct address и displacement lo/hi
2.2 Делаю struct для DA и DL и DH
*/

type Inst struct {
	Name      string
	Direction byte
	Size      byte
	Mode      byte
	Reg       byte
	RM        byte
}

type inst struct {
	name      string
	direction opDirection
	size      opSize
	mode      modeOffset
	reg       register
	rm        register
}

func new(i Inst) inst {
	return inst{
		name:      i.Name,
		direction: opDirection(i.Direction),
		size:      opSize(i.Size),
		mode:      modeOffset(i.Mode),
		reg:       register(i.Reg),
		rm:        register(i.RM),
	}
}

func (i inst) validate() error {
	return errors.Join(
		i.direction.validate(),
		i.size.validate(),
		i.mode.validate(),
		i.reg.validate(),
		i.rm.validate(),
	)
}

type InstEncoding interface {
	GetInst() Inst
}

type InstDirectAddrEncoding interface {
	InstEncoding

	DirectAddress() string
}

type InstDisplacementAddrEncoding interface {
	InstEncoding

	DispLo() byte
	DispHi() byte
}

func Parse(i InstEncoding) (string, error) {
	inst := new(i.GetInst())
	if err := inst.validate(); err != nil {
		return "", err
	}

	var result = ""
	if inst.mode == regOffset0 {
		result = parseRegEncoding(inst)
	} else {

	}
	return result, nil
}
