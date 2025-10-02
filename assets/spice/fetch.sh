#!/bin/bash

# Download SPICE kernels for scifisim project
# This script downloads the required kernels from NASA NAIF

set -e  # Exit on any error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Downloading SPICE kernels to $(pwd)..."

# Base URL for NAIF kernels
NAIF_BASE="https://naif.jpl.nasa.gov/pub/naif/generic_kernels"

# DE440s - Planetary ephemeris (small version)
echo "Downloading de440s.bsp..."
curl -L -o de440s.bsp "${NAIF_BASE}/spk/planets/de440s.bsp"

# NAIF leap seconds kernel
echo "Downloading naif0012.tls..."
curl -L -o naif0012.tls "${NAIF_BASE}/lsk/naif0012.tls"

# Planetary constants kernel
echo "Downloading pck00011.tpc..."
curl -L -o pck00011.tpc "${NAIF_BASE}/pck/pck00011.tpc"

# GM values for DE440
echo "Downloading gm_de440.tpc..."
curl -L -o gm_de440.tpc "${NAIF_BASE}/pck/gm_de440.tpc"

echo "Downloading jup365.bsp..."
curl -L -o jup365.bsp "${NAIF_BASE}/spk/satellites/jup365.bsp"

echo "Downloading mar099.bsp..."
curl -L -o mar099.bsp "${NAIF_BASE}/spk/satellites/mar099.bsp"

echo "Downloading nep095.bsp..."
curl -L -o nep095.bsp "${NAIF_BASE}/spk/satellites/nep095.bsp"

echo "Downloading plu060.bsp..."
curl -L -o plu060.bsp "${NAIF_BASE}/spk/satellites/plu060.bsp"

echo "Downloading sat457.bsp..."
curl -L -o sat457.bsp "${NAIF_BASE}/spk/satellites/sat457.bsp"

echo "Downloading ura184_part-1.bsp..."
curl -L -o ura184_part-1.bsp "${NAIF_BASE}/spk/satellites/ura184_part-1.bsp"

echo "Downloading ura184_part-2.bsp..."
curl -L -o ura184_part-2.bsp "${NAIF_BASE}/spk/satellites/ura184_part-2.bsp"

echo "Downloading ura184_part-3.bsp..."
curl -L -o ura184_part-3.bsp "${NAIF_BASE}/spk/satellites/ura184_part-3.bsp"

echo "All kernels downloaded successfully!"
echo ""
echo "Downloaded files:"
ls -lh *.bsp *.tls *.tpc
