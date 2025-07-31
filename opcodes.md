opcodes

0x01: 1 
     + 1 byte branch_type
     + 2 byte arg1
     + 2 byte arg2
     + jump offset (4 byte)
     + \0                                           - arg2 is offset to stored variable if branch type has bit 5 set to 1.
    - Branch types:
      - 1 => GE
      - 2 => LE
      - 3 => EQ
      - 4 => NE
      - 5 => GT
      - 6 => LT
0x02: 1                                             - Choice.
    + 1 byte n_choices
    + \0
    + choices:
          2 byte choice_arg1
        + \0 terminated string choice_arg2
        + 1 byte arg3
        + 2 byte arg4                               - Possibly jump target.
        + 1 byte arg5                               - unsure, but we check if this field == 3 to decide number of bytes in the rest of the choice.
        if zero:
          + 1 byte arg6
          + 2 byte arg7
          + 1 byte arg8
          + 1 byte arg9
          + 2 \0 bytes.
        if nonzero:
          if (arg5) >= 7:
            + \0 terminated string arg6
            + 2 bytes of padding
          + 6 bytes of padding.
0x03: 1                                             - Various load and store operations (like addition, subtraction and assigning random values)
     + 1 byte type                                  using the variable heap.
     + 2 byte arg1                                  - address into variable table
     + 1 byte arg2
     + 2 byte arg3
     + \0                                           - idk.
0x04: 1 byte.                                       - Does not advance IP until certain conditions are met.
0x05: 1
    + 1 byte arg1
    + \0                                            - idk.
0x06: 1 
    + 2 byte offset                                 - Unconditional jump to absolute offset within current script.
0x07: 1 
    + \0 terminated string arg1                     - Go to script named "string arg".wsc, not fully sure of the diff between 0x09.
0x08: 2 bytes.                                      - nop
0x09: 1 
    + \0 terminated string arg1                     - Go to script named "string arg".wsc
0x0A: 1                                             - Return
0x0B: 1 
    + 1 byte arg1
    + \0                                            - 
0x0C: 1 
    + 2 byte arg1
    + \0                                            -
0x0D: 1                                             - Debug print? One path outputs current opcode.
    + 2 byte arg1
    + 2 byte arg2
    + 4 bytes
    + \0
0x0E: 1
    + 1 byte arg1
    + \0
0x21: 1 
    + 1 byte arg1
    + 2 byte arg2
    + 1 byte arg3
    + 2 byte arg4
    + 4 byte arg5
    + \0 terminated string arg6                     - loop music file "arg6".ogg. 
0x22: 1
    + 1 byte arg1
    + 2 byte arg2
    + \0                                            - some file operation?
0x23 | 0x27:                                        - There's logic differentiating both opcodes, but idk what it's doing.
      1
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + 2 byte arg4
    + 1 byte arg5                                   - There's a check for 'e' here.
    + 1 byte arg5
    + \0 terminated string arg6                     - play "arg6".ogg character voice file.
0x24: 1 
    + \0                                        - Lots of sleep timers.
0x25: 1 A_B_CC_pp_D_EE_F
    + 1 byte arg1
    + 1 byte arg2
    + 2 byte arg3
    + 11 byte header
    + \0 terminated string arg1                     - Play sound effect file "arg1".
0x26: 1
    + 1 byte arg 1
    + \0
0x28: 1
    + 1 byte arg1
    + 1 byte arg2
    + 3 bytes of padding.
0x29: 1
    + 1 byte arg1
    + 2 byte arg2
    + 2 bytes of padding
0x30: 1
    + 1 byte arg1
    + 3 bytes of padding
0x31: 1
    + 1 byte arg1
    + \0
0x32: 1
    + 1 byte arg1
    + \0
0x33: 1
    + 2 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + \0
0x34: 1
    + 2 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + \0 terminated string arg4
0x41: 1                                             - Write textbox content, without a speaker.
    + 2 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + \0 terminated string arg4                     - arg4 is the string to print in textbox.
0x42: 1                                             - Write textbox content, with a speaker.
    + 2 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + 1 byte arg4
    + \0 terminated string textbox_title            - The speaker.
    + \0 terminated string text                     - The text to write.
0x43: 1
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + 1 byte arg4
    + \0 terminated string arg5                     - The file to load
0x44: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + \0
0x45: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + \0
0x46: 1 AA_BB_ppp_C_D_s
    + 9 byte header
    + \0 terminated string arg1                     - Load file named "string arg".png
0x47: 1
    + 1 byte arg1
    + \0
0x48: 1 A_BB_CC_DDDD_E_F
    + 11 byte header
    + \0
0x49: 1 + 2 byte arg1 + \0                          - arg1 seems to be a count of some kind.
0x4A: 1
    + 1 byte arg1
    + 2 byte arg2                                   - Possibly display loaded image with fade in effect.
    + 2 byte arg3
    + \0
0x4B: 1
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + 4 byte arg4
    + 2 byte arg5
    + 4 byte arg6
    + 4 byte arg7
    + \0
0x4C: 1
    + 1 byte arg1
    + 1 byte arg2
    + 4 byte arg3
    + \0
    + \0
0x4D: 1
    + 1 byte arg1
    + 1 byte arg2
    + 2 byte arg3
    + 2 byte arg4
    + 2 byte arg5
    + 2 byte arg6
    + 2 byte arg7
    + \0
0x4E: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + \0
0x4F: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + \0
0x50: 1
    + \0 terminated string arg1                     - load specified tbl file. seems unused
0x51: 1
    + 2 byte arg1
    + 2 byte arg2
    \0
0x52: 1
    + 1 byte arg1 (?)
    + \0
0x53: 1
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + \0 terminated string arg4                     - Print arg4 in message window.
0x54: 1
    + \0 terminated string arg1                     - load msk file.
0x55: 1
    + \0
0x56: 1
    + \0
0x57: 1
    + 2 byte arg1
    + 2 byte arg2
    + 4 byte arg3
    + \0
0x58: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + 2 byte arg4
    + 2 byte arg5
    + \0
0x59: 1
    + \0 terminated string arg1                     - load wip file.
0x60: 1
    + \0
0x61: 1
    + 1 byte arg1
    + \0 terminated string arg2                     - load movie file arg2.
0x62: 1
    + \0
0x63: 1
    + 1 byte arg1
    + 1 byte arg2
    + \0
0x64: 1
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + 2 byte arg4
    + \0
0x65: 1
    + 2 byte arg1
    + 2 byte arg2
    + \0
0x66: 1                                             - No 0 terminator? maybe for showing inlay?
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + 1 byte arg4
    + 2 byte arg5
    + 4 byte arg6
    + 2 byte arg7
    + 4 byte arg8
    + 4 byte arg9
0x67: 1
    + 1 byte arg1
    + 1 byte arg2
    + \0
    + 4 byte arg3
    + \0
0x68: 1
    + 2 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + 2 byte arg4
    + \0
0x69: 1
    + 1
    + \0
0x70: 1
    + 1 byte arg1
    + 1 byte arg2
    + \0
    + 4 byte arg3
    + \0
0x71: 1
    + \0 terminated string arg1
0x72: 1
    + \0
0x73: 1
    + 2 byte arg1
    + 2 byte arg2
    + 4 byte arg3
    + 1 byte arg4
    + \0 terminated string arg5
0x74: 1
    + 1 byte arg1
    + \0
0x75: 1
    + 2 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + 2 byte arg4
    + \0
0x76: 1
    + 2 byte arg1
    + 2 byte arg2
    + 4 byte arg3
    + 1 byte arg4
    + 1 byte arg5
    + 2 byte arg6
    + 4 byte arg7
    + \0
0x77: 1
    + 2 byte arg1
    + 2 byte arg2
    + 4 byte arg3
    + \0
0x78: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + 4 byte arg4
    + \0
0x79: 1
    + \0
0x81: 1
    + \0
    + \0
0x82: 1 
    + 2 byte arg1
    + \0
0x83: 1
    + \0
0x84: 1
    + \0
0x85: 1
    + 1 byte arg1
    + \0
0x86: 1
    + \0
    + \0
0x87: 1
    + 2 byte arg1
    + \0
0x88: 1
    + \0
    + \0
    + \0
0x89: 1
    + \0
0x8A: 1
    + \0
0x8B: 1
    + \0
0x8C: 1
    + 2 byte arg1
    + \0
0x8D: 1
    + \0
0x8E: 1
    + \0
0xA0: 1
    + 2 byte arg1
    + 2 byte arg2
    + 1 byte arg3
    + \0
0xA1: 1
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + 1 byte arg4
    + \0
0xA2: 1
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + \0
0xA3: 1
    + \0
    + 2 byte arg1
    + 2 byte arg2
    + \0
0xA4: 1
    + \0
    + 2 byte arg1
    + 2 byte arg2
    + \0
0xA5: 1
    + 1 byte arg1
    + \0
0xA6: 1
    + \0
0xA7: 1
    + \0
0xA8: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + \0
    + \0
    + \0
    + \0
    + 2 byte arg4
    + 2 byte arg5
    + 2 byte arg6
    + 2 byte arg7
    + \0
0xA9: 1
    + \0
0xAA: 1
    + 1 byte arg1
    + 1 byte arg2
    + \0
0xAB: 1
    + \0
0xAC: 1
    + \0
0xAD: 1
    + 1 byte arg1
    + 4 byte arg2
    + 4 byte arg3
    + \0
0xAE: 1
    + \0
0xB1: 1
    + 2 byte arg1
    + 2 byte arg2
    + \0
0xB2: 1                                             - load effect file
    + 1 byte arg1
    + \0
    + \0 terminated string arg2
0xB3: 1
    + \0
    + \0
0xB4: 1
    + \0
    + \0
    + 2 byte arg1
    + 2 byte arg2
    + 4 byte arg3
    + 1 byte arg4
    + \0
0xB5: 1
    + 1 byte arg1
    + 1 byte arg2
    + \0
    + \0
    + \0
    + \0
    + \0
0xB6: 1
    + 2 byte arg1
    + \0 terminated string arg2
0xB7: 1                                             - load wip or msk file?
    + 1 byte arg1
    + 2 byte arg2
    + 2 byte arg3
    + \0 terminated string arg4
0xB8: 1
    + 1 byte arg1
    + 1 byte arg2
    + \0
0xB9: 1
    + 1 byte arg1
    + 1 byte arg2
    + \0
0xBA: 1
    + 2 byte arg1
    + 2 byte arg2
    + 1 byte arg3
    + 1 byte arg4
    + 1 byte arg5
    + 1 byte arg6
    + 1 byte arg7
    + 2 byte arg8
    + \0 terminated string arg9
0xBB: 1
    + \0
0xBC: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + \0
0xBD: 1
    + 1 byte arg1
    + \0
0xBE: 1
    + 1 byte arg1
    + 1 byte arg2
    + \0
0xBF: 1
    + 1 byte arg1
    + 1 byte arg2
    + 1 byte arg3
    + 2 byte arg4
    + \0
0xE0: 1                                             - Set scene text? 
    + \0 terminated string arg1
0xE2: 1
    + \0
0xE3: 1
    + \0
0xE4: 1 
    + 1 byte arg1 
    + \0
0xE5: 1
    + \0
0xE6: 1
    + \0
    + \0
0xE7: 1
    + 2 byte arg1
    + \0
0xE8: 1
    + \0 terminated string arg1
0xE9: 1
    + \0
0xEA: 1                                             - Do something with wip or msk file.
    + 1 byte arg1
    + \0 terminated string arg2
0xEB: 1
    + \0
0xFF: 1                                             - exit.