"""Assemble the two ImageGen sprite rows into small, looping launcher GIFs."""
from pathlib import Path
from PIL import Image

root = Path(__file__).resolve().parent.parent
assets = root / 'launcher/assets/torizon-cats'
sheet = Image.open(assets / 'sprites.png').convert('RGBA')
width, height = sheet.width // 4, sheet.height // 2
for row, name in enumerate(['wave', 'wink']):
    frames = []
    for column in range(4):
        cell = sheet.crop((column * width + 8, row * height + 8,
                           (column + 1) * width - 8, (row + 1) * height - 8))
        # GIF has no partial alpha; use a clean card background when encoding its palette.
        background = Image.new('RGBA', cell.size, 'white')
        background.alpha_composite(cell)
        frames.append(background.convert('RGB').resize((184, 248), Image.Resampling.LANCZOS))
    frames[0].save(assets / f'{name}.gif', save_all=True, append_images=frames[1:],
                   duration=[600, 180, 320, 180], loop=0, disposal=2, optimize=True)
    frames[0].save(assets / f'{name}.png')
    print(f'{name}.gif: four frames, infinite loop')
