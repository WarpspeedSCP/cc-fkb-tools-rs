opcodes

0x01 : 1 
     + 1 byte branch type 
     + arg1 offset (2 byte)
     + arg2 (immediate or offset)
     + jump offset (4 byte)
     + \0                                       - arg2 is offset if branch type has bit 5 set to 1.
    - Branch types:
      - 1 => GE
      - 2 => LE
      - 3 => EQ
      - 4 => NE
      - 5 => GT
      - 6 => LT
0x02: 1                                         - Choice.
    + 1 
0x03 : 1                                        - Various load and store operations (like addition, subtraction and assigning random values) 
     + 1 byte type                                using the variable heap.
     + 2 byte arg1                              - address into variable table
     + 1 byte arg2
     + 2 byte arg3
     + \0                                       - idk.
0x06 : 1 + 2 byte offset                        - Unconditional jump to absolute offset within current script.
0x07 : 1 + string arg + \0                      - Go to script named "string arg".wsc, not fully sure of the diff between 0x09.
0x09 : 1 + string arg + \0                      - Go to script named "string arg".wsc
0x0A : 1                                        - Return
0x22 : 1 + 1 byte arg1 + 2 byte arg2 + \0       - some file operation?
0x46 : 1 + 9 byte header + string arg + \0      - Load file named "string arg".png
0x49 : 1 + 2 byte arg1 + \0                     - arg1 seems to be a count of some kind.
0x4A : 1 + 6 byte body                          - Possibly display loaded image with fade in effect.
0x82 : 1 + 2 byte operand + \0                  - 1 byte padding?
0xE4 : 1 + 1 byte arg1 + \0
