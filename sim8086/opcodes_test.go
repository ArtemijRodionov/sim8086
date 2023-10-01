package sim8086

import "testing"
import "fmt"

func TestParseInstruction(t *testing.T) {
	tests := []struct {
		first  byte
		second byte
		result string
	}{
		{0x89, 0xd9, "mov cx, bx"},
	}

	for _, test := range tests {
		inst := ParseIntruction(test.first, test.second)
		if fmt.Sprint(inst) != test.result {
			t.Errorf("fmt.Sprint(Instruction{%x, %x}) != %s", test.first, test.second, test.result)
		}
	}
}
