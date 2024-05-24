import itertools

file = open("super_mario_world.sfc", "rb")
rom = bytearray(file.read())
file.close()

sum = 0
for i in range(len(rom)):
    sum = (sum+rom[i]) & 0xFFFF

print(hex(sum))
print(hex(sum^0xFFFF) )