"""Encode the ImageGen atlas as two transparent, eight-frame looping GIFs."""
from pathlib import Path
from PIL import Image

root = Path(__file__).resolve().parent.parent
assets = root / 'launcher/assets/torizon-cats'
sheet = Image.open(assets / 'sprites.png').convert('RGBA')
width, height = sheet.width // 4, sheet.height // 4
for group, name in enumerate(['wave', 'wink']):
    frames = []
    palette = sheet.convert('RGB').quantize(colors=255)
    for index in range(8):
        row, column = divmod(group * 8 + index, 4)
        cell = sheet.crop((column * width, row * height,
                           (column + 1) * width, (row + 1) * height))
        cell = cell.resize((256, 256), Image.Resampling.LANCZOS)
        if index == 0:
            cell.save(assets / f'{name}.png')
        frame = cell.convert('RGB').quantize(palette=palette, dither=Image.Dither.NONE)
        transparent = cell.getchannel('A').point(lambda alpha: 255 if alpha < 128 else 0)
        frame.paste(255, mask=transparent)
        frame.info['transparency'] = 255
        frames.append(frame)
    frames[0].save(assets / f'{name}.gif', save_all=True, append_images=frames[1:],
                   duration=[280, 100, 100, 120, 160, 120, 100, 180],
                   loop=0, disposal=2, transparency=255, background=255, optimize=False)
    result = Image.open(assets / f'{name}.gif')
    assert result.n_frames == 8 and result.info['loop'] == 0
    for index in range(result.n_frames):
        result.seek(index)
        assert result.convert('RGBA').getpixel((0, 0))[3] == 0
    print(f'{name}.gif: eight frames, infinite loop, transparent corners')
