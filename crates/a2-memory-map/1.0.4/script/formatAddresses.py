'''Used to change the keys in `map.json`.
Options are:
* pos - positive decimal addresses
* neg - signed decimal addresses (0xc000 or above go negative)
* hex - padded hexidecimal addresses'''

import json
import sys

if len(sys.argv)!=2:
    print('provide one of `pos`,`neg`,`hex`')
    exit()

if sys.argv[1] not in ['pos','neg','hex']:
    print('provide one of `pos`,`neg`,`hex`')
    exit()

with open("specialAddresses.json") as f:
    obj = json.loads(f.read())

newobj = {}

for key in obj:
    try:
        addr = int(key,10)
    except ValueError:
        addr = int(key,16)
    if sys.argv[1]=='pos':
        if addr<0:
            addr += 0x10000
        newkey = str(addr)
    elif sys.argv[1]=='neg':
        if addr>=0xc000:
            addr -= 0x10000
        newkey = str(addr)
    elif sys.argv[1]=='hex':
        if addr<0:
            addr += 0x10000
        newkey = format(addr,"#06x")
    newobj[newkey] = obj[key]

with open("specialAddresses.json","w") as f:
    f.write(json.dumps(newobj,sort_keys=True,indent=4))
        
    