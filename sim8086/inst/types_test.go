package inst

import (
	"testing"
)

type A byte

func (a A) Mode() ModeOffset {
	return memOffset0
}
func (a A) Name() string {
	return "test"
}

func (a A) W() OpSize {
	return opWord
}
func (a A) Reg() Register {
	return alax
}
func (a A) RM() Register {
	return alax
}

func TestTypes(t *testing.T) {
	a := A(0)
	inst, err := New(a).Parse()
	if err != nil {
		t.Errorf("Got error %s", err)
	}

	if inst != "test ax, ax" {
		t.Errorf("Got %s", inst)
	}
}
