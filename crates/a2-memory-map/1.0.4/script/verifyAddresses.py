'''This is a way of error checking addresses.
It goes through each entry and displays the disassembly
near the given address.  The user can then check to
see if the description and disassembly match up.'''

import json
import sys
import re

if len(sys.argv)!=2:
    print('provide path to disassembly text')
    exit()

with open("../src/map.json") as f:
    obj = json.loads(f.read())

with open(sys.argv[1]) as f:
    dasm = f.read()

def findDASM(key):
    # first try to find ML addr
    for i,line in enumerate(dasm.splitlines()):
        if line[0:4].lower()==key[2:].lower():
            for j in range(i-2,i+3):
                print(dasm.splitlines()[j])
            return
    # not found, try to find an equate
    trimmedKey = key[2:]
    if trimmedKey[0]=='0' and trimmedKey[1]=='0':
        trimmedKey = key[4:]
    patt = 'EQU\\s+\\$' + trimmedKey.upper()
    for i,line in enumerate(dasm.splitlines()):
        matches = re.search(patt,line)
        if matches!=None:
            print(line)

for key in obj:
    print(key)
    print(obj[key])
    findDASM(key)
    input()
    