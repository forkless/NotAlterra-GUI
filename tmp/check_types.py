import re
c = open('gvas-files/_extracted/savegame_1.sav','rb').read()
s = open('gvas-files/_extracted/savegame_0.sav','rb').read()
types = ['FloatProperty','IntProperty','DoubleProperty','StructProperty',
         'StrProperty','ByteProperty','BoolProperty','NameProperty','ArrayProperty',
         'TextProperty','Int64Property','Int8Property','SetProperty','MapProperty']
print("Survival:")
for t in types:
    pat = t.encode() + b'\x00'
    n = len(re.findall(pat, s))
    if n > 0:
        print(f'  {t}: {n}')
print("\nCreative:")
for t in types:
    pat = t.encode() + b'\x00'
    n = len(re.findall(pat, c))
    if n > 0:
        print(f'  {t}: {n}')
