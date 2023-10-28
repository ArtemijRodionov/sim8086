package inst

var effAddrOffset0Encoding = map[Register]string{
	alax: "bx + si",
	clcx: "bx + di",
	dldx: "bp + si",
	blbx: "bp + di",
	ahsp: "si",
	chbp: "di",
	// dhsi TODO direct address
	bhdi: "bx",
}
