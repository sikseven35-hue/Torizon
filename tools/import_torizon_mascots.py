"""Import the user's GIFs, removing their edge-connected white background.

The closed ink contour protects white fur. Frame timing, canvas size and the loop
are retained; each frame is fully disposed so transparent pixels leave no trails.
Requires Pillow, NumPy and OpenCV for this development-only import.
"""
import argparse
import json
from pathlib import Path
import cv2
import numpy as np
from PIL import GifImagePlugin, Image


def cutout(image, cropped_bottom=False, kernel_size=15):
    rgb = np.asarray(image.convert('RGB'))
    gray = cv2.cvtColor(rgb, cv2.COLOR_RGB2GRAY)
    ink = (gray < 195).astype(np.uint8)
    barrier = cv2.morphologyEx(ink, cv2.MORPH_CLOSE,
                             cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (kernel_size, kernel_size)))
    # A torso cropped by the canvas has an open contour at that edge. Connect
    # its outline endpoints along the crop boundary before flooding the outside.
    for edge in (barrier[0, :], barrier[-1, :], barrier[:, 0], barrier[:, -1]):
        points = np.flatnonzero(edge)
        if len(points) > 1:
            edge[points[0]:points[-1] + 1] = 1
    if cropped_bottom:
        # The supplied close-up has a narrow white margin below a cropped torso.
        points = np.flatnonzero(np.any(barrier[-24:, :], axis=0))
        if len(points) > 1:
            left, right = points[0], points[-1]
            barrier[-24:, left] = barrier[-24:, right] = 1
            barrier[-1, left:right + 1] = 1
    regions = np.pad(barrier, 1)
    cv2.floodFill(regions, None, (0, 0), 2)
    mask = (regions[1:-1, 1:-1] != 2).astype(np.uint8)
    count, labels, stats, _ = cv2.connectedComponentsWithStats(mask, connectivity=8)
    if count <= 1:
        raise ValueError('No character contour found')
    largest = 1 + np.argmax(stats[1:, cv2.CC_STAT_AREA])
    alpha = np.where(labels == largest, 255, 0).astype(np.uint8)
    rgba = Image.fromarray(np.dstack((rgb, alpha)), 'RGBA')
    return rgba, int(np.count_nonzero(alpha))


def import_gif(source, destination):
    original = Image.open(source)
    rgba_frames, durations, areas = [], [], []
    loop = original.info.get('loop', 0)
    for index in range(original.n_frames):
        original.seek(index)
        frame, area = cutout(original, cropped_bottom=destination.stem == 'wink')
        if areas and area < areas[0] * .65:
            for kernel_size in (19, 23, 27, 31):
                frame, area = cutout(original, cropped_bottom=destination.stem == 'wink',
                                     kernel_size=kernel_size)
                if area >= areas[0] * .65:
                    break
        rgba_frames.append(frame)
        areas.append(area)
        durations.append(original.info.get('duration', 100))
    if min(areas) < np.median(areas) * .5:
        raise ValueError(f'Character contour leaked: areas {min(areas)}..{max(areas)}')
    # One shared palette avoids color flicker. Index 255 is exclusively transparent.
    samples = Image.new('RGB', (128 * len(rgba_frames), 128))
    for index, frame in enumerate(rgba_frames):
        samples.paste(frame.convert('RGB').resize((128, 128)), (128 * index, 0))
    palette = samples.quantize(colors=255)
    frames = []
    for rgba in rgba_frames:
        frame = rgba.convert('RGB').quantize(palette=palette, dither=Image.Dither.NONE)
        frame.paste(255, mask=rgba.getchannel('A').point(lambda a: 255 if a == 0 else 0))
        frames.append(frame)
    # Pillow's normal multi-frame save merges identical poses after the background
    # noise is removed. Write full frames so even repeated source frames survive.
    with destination.open('wb') as output:
        for block in GifImagePlugin._get_global_header(
                frames[0], dict(loop=loop, background=255, transparency=255)):
            output.write(block)
        for frame, duration in zip(frames, durations):
            GifImagePlugin._write_frame_data(output, frame, (0, 0),
                                            dict(duration=duration, disposal=2, transparency=255))
        output.write(b';')
    rgba_frames[0].save(destination.with_suffix('.png'))
    result = Image.open(destination)
    assert result.size == original.size and result.n_frames == original.n_frames
    actual_durations = []
    for index in range(result.n_frames):
        result.seek(index)
        actual_durations.append(result.info.get('duration', 100))
        alpha = result.convert('RGBA').getchannel('A')
        assert alpha.getpixel((0, 0)) == 0
        assert alpha.getextrema() == (0, 255)
    assert actual_durations == durations and result.info.get('loop') == loop
    return dict(source=source.name, output=destination.name, size=result.size,
                frames=result.n_frames, duration_ms=sum(durations), loop=loop,
                foreground_pixels=[min(areas), max(areas)])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--wave', type=Path, required=True)
    parser.add_argument('--wink', type=Path, required=True)
    parser.add_argument('--output', type=Path,
                        default=Path(__file__).resolve().parent.parent / 'launcher/assets/torizon-cats')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    results = [import_gif(source, args.output / f'{name}.gif')
               for source, name in [(args.wave, 'wave'), (args.wink, 'wink')]]
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
