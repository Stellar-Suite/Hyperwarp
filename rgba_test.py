from PIL import Image
import sys
filepath = sys.argv[1]
width = 240
height = 160
# open raw rgba
with open(filepath, 'rb') as f:
    raw = f.read()
    # convert to PIL Image
    img = Image.frombytes('RGBA', (width, height), raw)
    # save
    img.save(filepath + '.png')