package inst

type Inst = struct {
	Direction byte
	Size      byte
	Mode      byte
	Reg       byte
	RM        byte
	Name      string
}

type InstReader interface {
	Inst() Inst
	DispLo() byte
	DispHi() byte
}

type DirectAddrReader interface {
	DirectAddress() int16
}

func Read(i InstReader) (string, error) {
	inst := new(i.Inst())
	if err := inst.validate(); err != nil {
		return "", err
	}

	if inst.mode == regOffset0 {
		return parseRegEncoding(inst), nil
	}

	if isDirectAddress(inst.mode, inst.rm) {
		return "da", nil
	}

	return "", nil
}
