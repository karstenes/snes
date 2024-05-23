import itertools

file = open("super_metroid.sfc", "rb")
rom = bytearray(file.read())
file.close()

sum = 0
for byte in itertools.batched(rom, 2):
    sum += (byte[0] << 8 | byte[1])
print(sum % 65536)