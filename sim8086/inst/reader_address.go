package inst

var effAddrOffset0Encoding = map[register]string{
	alax: "bx + si",
	clcx: "bx + di",
	dldx: "bp + si",
	blbx: "bp + di",
	ahsp: "si",
	chbp: "di",
	dhsi: "bp",
	bhdi: "bx",
}

func isDirectAddress(m modeOffset, r register) bool {
	return m == memOffset0 && r == dhsi
}
