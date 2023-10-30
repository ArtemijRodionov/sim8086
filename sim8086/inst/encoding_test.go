package inst

import (
	"testing"
)

type A byte

func (a A) GetInst() Inst {
	return Inst{
		Name:      "test",
		Direction: 0x0,
		Size:      0x0,
		Mode:      0x3,
		Reg:       0x0,
		RM:        0x0,
	}
}

func TestTypes(t *testing.T) {
	var a A
	inst, err := Parse(a)
	if err != nil {
		t.Errorf("Got error %s", err)
	}

	if inst != "test al, al" {
		t.Errorf("Got %s", inst)
	}
}
