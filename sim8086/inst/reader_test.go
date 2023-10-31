package inst

import (
	"testing"
)

type A byte

func (a A) Inst() Inst {
	return Inst{
		Direction: 0x0,
		Size:      0x0,
		Mode:      0x3,
		Reg:       0x0,
		RM:        0x0,
		Name:      "test",
	}
}

func (a A) DispHi() byte {
	return 0
}

func (a A) DispLo() byte {
	return 0
}

func TestTypes(t *testing.T) {
	var a A
	inst, err := Read(a)
	if err != nil {
		t.Errorf("Got error %s", err)
	}

	if inst != "test al, al" {
		t.Errorf("Got %s", inst)
	}
}
