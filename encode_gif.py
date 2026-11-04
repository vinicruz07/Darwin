import os
from PIL import Image

def main():
    raw_path = "target/gif_frames/frames.raw"
    if not os.path.exists(raw_path):
        print("Arquivo raw nao encontrado!")
        return

    w = 640
    h = 360
    frame_size = w * h * 3

    file_size = os.path.getsize(raw_path)
    total_frames = file_size // frame_size

    print(f">>> Compactando {total_frames} frames em GIF animado de alta qualidade...")

    images = []
    with open(raw_path, "rb") as f:
        for i in range(total_frames):
            data = f.read(frame_size)
            img = Image.frombytes("RGB", (w, h), data)
            # Converte para paleta indexada otimizada (P) para GIF leve e de alta fidelidade
            img_p = img.quantize(colors=128, method=Image.Quantize.MEDIANCUT)
            images.append(img_p)
            if (i + 1) % 50 == 0:
                print(f"  Processados {i + 1}/{total_frames} frames...")

    out_gif = "darwin_racing_demo.gif"
    # Duração de 33ms por frame = ~30 FPS suave
    images[0].save(
        out_gif,
        save_all=True,
        append_images=images[1:],
        duration=33,
        loop=0,
        optimize=True,
    )
    print(f">>> Sucesso! Animacao salva em: {os.path.abspath(out_gif)}")

if __name__ == "__main__":
    main()
