#! /usr/bin/env python3

# Fix broadcasting bug: ensure thickness arrays are HxW before multiplying by taper (HxW).

from PIL import Image, ImageDraw, ImageFont
import numpy as np
import math

W, H = 2048, 1024
UV_ORIGIN = "TOP_LEFT"
SEAM_OFFSET_DEG = 0
DLON_DEG_MAJOR = 60
DLAT_DEG_MAJOR = 30
DLON_DEG_MINOR = 15
DLAT_DEG_MINOR = 15
AA_WIDTH_MAJOR = 3.8
AA_WIDTH_MINOR = 3.1
MAJOR_EQ_SCALE  = 2.0
MAJOR_PM_SCALE  = 1.8
POLE_TAPER = True

RGB_WHITE = np.array([255,255,255], dtype=np.float32)
SURF_SKY  = np.array([0x5A,0xA7,0xFF], dtype=np.float32)
SURF_GND  = np.array([0x8B,0x6A,0x3B], dtype=np.float32)
SPACE_TOP = np.array([0x10,0x10,0x10], dtype=np.float32)
SPACE_BOT = np.array([0x40,0x40,0x40], dtype=np.float32)
GRID_COLOR = RGB_WHITE
GRID_ALPHA = 1.0
LABEL_FONT = "DejaVuSans.ttf"
LABEL_FONT_BOLD = "DejaVuSans-Bold.ttf"
LABEL_SIZE = 60
LABEL_SIZE_BOLD = 64
LABEL_STROKE = 4

def make_background(surface_mode=True):
    img = np.zeros((H, W, 3), dtype=np.float32)
    v = (np.arange(H) + 0.5) / H
    if UV_ORIGIN.upper() == "TOP_LEFT":
        phi = math.pi * (0.5 - v)
    else:
        phi = math.pi * (v - 0.5)
    sky_mask = (phi >= 0).astype(np.float32)[:, None, None]
    if surface_mode:
        img[:] = SURF_GND
        img = img * (1 - sky_mask) + SURF_SKY[None,None,:] * sky_mask
    else:
        img[:] = SPACE_BOT
        img = img * (1 - sky_mask) + SPACE_TOP[None,None,:] * sky_mask
    return img

def line_mask_equirect():
    x = (np.arange(W) + 0.5)[None, :].astype(np.float64)  # 1xW
    y = (np.arange(H) + 0.5)[:, None].astype(np.float64)  # Hx1
    u = x / W
    v = y / H
    lam = 2*math.pi*u - math.pi + math.radians(SEAM_OFFSET_DEG)
    lam = (lam + math.pi) % (2*math.pi) - math.pi
    if UV_ORIGIN.upper() == "TOP_LEFT":
        phi = math.pi * (0.5 - v)
    else:
        phi = math.pi * (v - 0.5)

    px_per_rad_x = W / (2*math.pi)
    px_per_rad_y = H / math.pi

    def dist_to_grid(angle, step_rad):
        k = np.round(angle / step_rad)
        return np.abs(angle - k * step_rad)

    dlon_major = dist_to_grid(lam, math.radians(DLON_DEG_MAJOR))      # HxW
    dlat_major = dist_to_grid(phi, math.radians(DLAT_DEG_MAJOR))      # HxW
    dlon_minor = dist_to_grid(lam, math.radians(DLON_DEG_MINOR)) if DLON_DEG_MINOR else np.full_like(dlon_major, 1e9)
    dlat_minor = dist_to_grid(phi, math.radians(DLAT_DEG_MINOR)) if DLAT_DEG_MINOR else np.full_like(dlat_major, 1e9)

    # Start with scalars and broadcast to HxW explicitly
    w_lon_major = np.full_like(dlon_major, AA_WIDTH_MAJOR)
    w_lat_major = np.full_like(dlat_major, AA_WIDTH_MAJOR)

    pm_emph = (np.abs(lam) < 1e-6) | (np.abs(np.abs(lam) - math.pi/2) < 1e-6)
    w_lon_major = w_lon_major * (1 + (MAJOR_PM_SCALE - 1) * pm_emph)

    eq_emph = (np.abs(phi) < 1e-6)
    w_lat_major = w_lat_major * (1 + (MAJOR_EQ_SCALE - 1) * eq_emph)

    if POLE_TAPER:
        taper = np.clip(np.cos(phi), 0.2, 1.0)  # HxW
        w_lon_major = w_lon_major * taper
        w_lat_major = w_lat_major * taper
        w_lon_minor = np.full_like(dlon_major, AA_WIDTH_MINOR) * taper
        w_lat_minor = np.full_like(dlat_major, AA_WIDTH_MINOR) * taper
    else:
        w_lon_minor = np.full_like(dlon_major, AA_WIDTH_MINOR)
        w_lat_minor = np.full_like(dlat_major, AA_WIDTH_MINOR)

    dist_lon_px_major = dlon_major * px_per_rad_x
    dist_lat_px_major = dlat_major * px_per_rad_y
    dist_lon_px_minor = dlon_minor * px_per_rad_x
    dist_lat_px_minor = dlat_minor * px_per_rad_y

    def aa_from(dist_px, width_px):
        t = 1.0 - np.clip(dist_px / (width_px + 1e-9), 0.0, 1.0)
        return t * t

    t_lon_major = aa_from(dist_lon_px_major, w_lon_major)
    t_lat_major = aa_from(dist_lat_px_major, w_lat_major)
    t_lon_minor = aa_from(dist_lon_px_minor, w_lon_minor)
    t_lat_minor = aa_from(dist_lat_px_minor, w_lat_minor)

    t = np.maximum(np.maximum(t_lon_major, t_lat_major),
                   np.maximum(t_lon_minor, t_lat_minor))
    return t

def compose_theme_with_labels(surface_mode=True):
    bg = make_background(surface_mode)
    grid = line_mask_equirect()[..., None]
    out = bg * (1 - GRID_ALPHA*grid) + GRID_COLOR[None,None,:] * (GRID_ALPHA*grid)
    out = np.clip(out, 0, 255).astype(np.uint8)
    img = Image.fromarray(out).convert("RGBA")

    draw = ImageDraw.Draw(img)
    try:
        font = ImageFont.truetype(LABEL_FONT, LABEL_SIZE)
        font_b = ImageFont.truetype(LABEL_FONT_BOLD, LABEL_SIZE_BOLD)
    except:
        font = ImageFont.load_default()
        font_b = font

    def draw_text_center(x, y, s, bold=False):
        f = font_b if bold else font
        bbox = draw.textbbox((0,0), s, font=f, stroke_width=LABEL_STROKE)
        w = bbox[2]-bbox[0]; h = bbox[3]-bbox[1]
        draw.text((x - w/2, y - h/2), s, font=f,
                  fill=(255,255,255,255),
                  stroke_width=LABEL_STROKE,
                  stroke_fill=(0,0,0,255))

    # Cardinals
    if False:
        draw_text_center(W/2, 24, "N", bold=True)
        draw_text_center(W/2, H-24, "S", bold=True)
        draw_text_center(3*W/4, H/2, "E", bold=True)
        draw_text_center(W/4, H/2, "W", bold=True)

    # Longitudes every 30° along equator
    for lon in range(-150, 181, 30):
        u = (lon + 180.0) / 360.0
        x = u * W
        if lon == -180 or lon == 180:
            # s = "180°"
            s = "180"
        elif lon == 0:
            # s = "0°"
            s = "0"
        elif lon > 0:
            # s = f"{lon}°E"
            s = f"+{lon}"
        else:
            # s = f"{-lon}°W"
            s = f"{lon}"
        draw_text_center(x, H/2 - 18, s)
        if lon == 180:
            draw_text_center(0, H/2 - 18, s)

    # Latitudes every 15° on right edge
    for lat in range(-75, 90, 15):
        v = (0.5 - lat/180.0) if UV_ORIGIN.upper()=="TOP_LEFT" else (lat/180.0 + 0.5)
        y = v * H
        if lat == 0:
            # s = "0°"
            s = "0"
        elif lat > 0:
            # s = f"{lat}°N"
            s = f"+{lat}"
        else:
            # s = f"{-lat}°S"
            s = f"{lat}"
        draw_text_center(W - 80, y, s)
        draw_text_center(W/2 - 80, y, s)

    return img

def make_vignette(size=512, inner=0.90, outer=0.99):
    S = size
    x = (np.arange(S) + 0.5)[None, :]
    y = (np.arange(S) + 0.5)[:, None]
    cx = cy = S / 2.0
    r = np.sqrt((x - cx)**2 + (y - cy)**2) / (S/2.0)
    t = np.clip((r - inner) / (outer - inner + 1e-9), 0.0, 1.0)
    alpha = (t * 255).astype(np.uint8)
    rgba = np.dstack([np.zeros((S,S,3), dtype=np.uint8), alpha])
    return Image.fromarray(rgba)

surf = compose_theme_with_labels(True)
space = compose_theme_with_labels(False)
vig = make_vignette(512, 0.90, 0.99)

p_surf = "../assets/tex/navball_surface_2048x1024.png"
p_space = "../assets/tex/navball_space_2048x1024.png"
p_vig = "../assets/tex/vignette_512.png"

surf.save(p_surf)
space.save(p_space)
vig.save(p_vig)

(p_surf, p_space, p_vig)
