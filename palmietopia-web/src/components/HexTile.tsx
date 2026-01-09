interface HexTileProps {
  q: number;
  r: number;
  terrain: string;
  size: number;
}

const TERRAIN_COLORS: Record<string, string> = {
  Grassland: "#7CB342",
  Forest: "#2E7D32",
  Mountain: "#757575",
  Water: "#1976D2",
  Desert: "#F57C00",
};

// Darker shades for depth effect
const TERRAIN_DARK: Record<string, string> = {
  Grassland: "#558B2F",
  Forest: "#1B5E20",
  Mountain: "#424242",
  Water: "#0D47A1",
  Desert: "#E65100",
};

export function HexTile({ q, r, terrain, size }: HexTileProps) {
  // Convert axial coordinates to pixel coordinates (pointy-top)
  const x = size * (Math.sqrt(3) * q + (Math.sqrt(3) / 2) * r);
  const y = size * ((3 / 2) * r);

  // Generate hex polygon points (pointy-top orientation)
  const points = [];
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 180) * (60 * i - 30);
    const px = x + size * Math.cos(angle);
    const py = y + size * Math.sin(angle);
    points.push(`${px},${py}`);
  }

  const color = TERRAIN_COLORS[terrain] || "#CCCCCC";
  const darkColor = TERRAIN_DARK[terrain] || "#999999";

  return (
    <g>
      {/* Base tile with gradient for depth */}
      <defs>
        <linearGradient id={`grad-${q}-${r}`} x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor={color} />
          <stop offset="100%" stopColor={darkColor} />
        </linearGradient>
      </defs>
      <polygon
        points={points.join(" ")}
        fill={`url(#grad-${q}-${r})`}
        stroke="#1a1a1a"
        strokeWidth="1.5"
        className="transition-all hover:brightness-125 cursor-pointer"
      />
      {/* Subtle highlight on top edge */}
      <line
        x1={points[4].split(",")[0]}
        y1={points[4].split(",")[1]}
        x2={points[5].split(",")[0]}
        y2={points[5].split(",")[1]}
        stroke="rgba(255,255,255,0.2)"
        strokeWidth="1"
      />
    </g>
  );
}
