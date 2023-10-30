package inst

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