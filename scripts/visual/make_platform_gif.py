#!/usr/bin/env python3
"""Build a labelled GIF from screenshots captured by real CI runners."""

from pathlib import Path
import sys

from PIL import Image, ImageDraw, ImageFont


ORDER = ("web", "android", "ios", "macos", "windows")
LABELS = {
    "web": "Web",
    "android": "Android",
    "ios": "iOS / iPadOS",
    "macos": "macOS",
    "windows": "Windows",
}


def fit(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    image = image.convert("RGB")
    image.thumbnail(size, Image.Resampling.LANCZOS)
    canvas = Image.new("RGB", size, "#0b1020")
    canvas.paste(image, ((size[0] - image.width) // 2, (size[1] - image.height) // 2))
    return canvas


def main() -> int:
    source = Path(sys.argv[1] if len(sys.argv) > 1 else "screenshots")
    output = Path(sys.argv[2] if len(sys.argv) > 2 else "tektalk-platform-demo.gif")
    font = ImageFont.load_default(size=28)
    frames: list[Image.Image] = []

    for platform in ORDER:
        images = sorted(source.rglob(f"{platform}*.png"))
        if not images:
            raise SystemExit(f"Missing screenshot for {platform}")
        for path in images:
            shot = fit(Image.open(path), (1280, 720))
            frame = Image.new("RGB", (1280, 790), "#0b1020")
            frame.paste(shot, (0, 70))
            draw = ImageDraw.Draw(frame)
            label = f"TEKtalk · {LABELS[platform]} · real CI capture"
            draw.text((32, 20), label, fill="#f8fafc", font=font)
            frames.append(frame)

    output.parent.mkdir(parents=True, exist_ok=True)
    frames[0].save(
        output,
        save_all=True,
        append_images=frames[1:],
        duration=1800,
        loop=0,
        optimize=True,
    )
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
