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
		inst := NewInstruction(test.first, test.second)
		if result := fmt.Sprint(inst); result != test.result {
			t.Errorf("'%s' != '%s' for Instruction{%x, %x}", result, test.result, test.first, test.second)
		}
	}
}
