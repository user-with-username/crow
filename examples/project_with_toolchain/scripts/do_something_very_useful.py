import random
import struct

def generate_data(path: str, items_count: int):
    with open(path, 'wb') as f:
        for _ in range(items_count):
            number: int = random.randint(-1000, 1000)
            weight: float = random.random() * 10.0 # lol
            f.write(struct.pack('if', number, weight))

generate_data('./data.bin', 1000)
