package mov

import "testing"
import "fmt"

func TestParseInstruction(t *testing.T) {
	tests := []struct {
		first  byte
		second byte
		result string
	}{
		{0x89, 0xd9, "mov cx, bx"},
		{0x88, 0xe5, "mov ch, ah"},
		{0x89, 0xda, "mov dx, bx"},
		{0x89, 0xde, "mov si, bx"},
		{0x89, 0xfb, "mov bx, di"},
		{0x88, 0xc8, "mov al, cl"},
		{0x88, 0xed, "mov ch, ch"},
		{0x89, 0xc3, "mov bx, ax"},
		{0x89, 0xf3, "mov bx, si"},
		{0x89, 0xfc, "mov sp, di"},
		{0x89, 0xc5, "mov bp, ax"},
		{0xb1, 0x0c, "mov cl, 12"},
	}

	for _, test := range tests {
		t.Run(test.result, func(t *testing.T) {
			inst := NewInstruction(test.first, test.second)
			if result := fmt.Sprint(inst); result != test.result {
				t.Errorf("'%s' != '%s' for Instruction{%x, %x}", result, test.result, test.first, test.second)
			}
		})
	}
}
