interface HexTileProps {
  q: number;
  r: number;
  terrain: string;
  size: number;
  onClick?: () => void;
  isHighlighted?: boolean;
  isSelected?: boolean;
  visibilityState?: "visible" | "explored" | "unexplored";
}

const TERRAIN_COLORS: Record<string, string> = {
  Grassland: "#7CB342",
  Forest: "#2E7D32",
  Mountain: "#757575",
  Water: "#1976D2",
  Desert: "#F57C00",
};

const TERRAIN_DARK: Record<string, string> = {
  Grassland: "#558B2F",
  Forest: "#1B5E20",
  Mountain: "#424242",
  Water: "#0D47A1",
  Desert: "#E65100",
};

export function HexTile({ q, r, terrain, size, onClick, isHighlighted, isSelected, visibilityState = "visible" }: HexTileProps) {
  const x = size * (Math.sqrt(3) * q + (Math.sqrt(3) / 2) * r);
  const y = size * ((3 / 2) * r);

  const points = [];
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 180) * (60 * i - 30);
    const px = x + size * Math.cos(angle);
    const py = y + size * Math.sin(angle);
    points.push(`${px},${py}`);
  }

  const color = TERRAIN_COLORS[terrain] || "#CCCCCC";
  const darkColor = TERRAIN_DARK[terrain] || "#999999";
  const gradientId = `grad-${q}-${r}`;

  const renderTerrainDecoration = () => {
    const s = size * 0.3;
    
    switch (terrain) {
      case "Grassland":
        return (
          <g>
            {[-0.2, 0, 0.2].map((offset, i) => (
              <path
                key={i}
                d={`M${x + offset * size},${y + size * 0.15} 
                    Q${x + offset * size - 3},${y - size * 0.1} ${x + offset * size},${y - size * 0.2}
                    Q${x + offset * size + 3},${y - size * 0.1} ${x + offset * size},${y + size * 0.15}`}
                fill="none"
                stroke="rgba(0,100,0,0.3)"
                strokeWidth="1.5"
              />
            ))}
          </g>
        );
      
      case "Forest":
        return (
          <g>
            {[
              { ox: -s * 0.8, oy: s * 0.3 },
              { ox: s * 0.8, oy: s * 0.3 },
              { ox: 0, oy: -s * 0.5 },
            ].map((pos, i) => (
              <g key={i}>
                <polygon
                  points={`${x + pos.ox},${y + pos.oy - s * 0.8} 
                           ${x + pos.ox - s * 0.5},${y + pos.oy + s * 0.4} 
                           ${x + pos.ox + s * 0.5},${y + pos.oy + s * 0.4}`}
                  fill="#1a472a"
                  opacity="0.7"
                />
                <rect
                  x={x + pos.ox - 2}
                  y={y + pos.oy + s * 0.3}
                  width="4"
                  height={s * 0.3}
                  fill="#5d4037"
                  opacity="0.6"
                />
              </g>
            ))}
          </g>
        );
      
      case "Mountain":
        return (
          <g>
            <polygon
              points={`${x},${y - size * 0.4} 
                       ${x - size * 0.35},${y + size * 0.25} 
                       ${x + size * 0.35},${y + size * 0.25}`}
              fill="#5d5d5d"
              opacity="0.8"
            />
            <polygon
              points={`${x},${y - size * 0.4} 
                       ${x - size * 0.12},${y - size * 0.15} 
                       ${x + size * 0.12},${y - size * 0.15}`}
              fill="#ffffff"
              opacity="0.7"
            />
            <polygon
              points={`${x - size * 0.25},${y + size * 0.1} 
                       ${x - size * 0.45},${y + size * 0.35} 
                       ${x - size * 0.05},${y + size * 0.35}`}
              fill="#4a4a4a"
              opacity="0.6"
            />
          </g>
        );
      
      case "Water":
        return (
          <g>
            {[-0.2, 0, 0.2].map((offset, i) => (
              <path
                key={i}
                d={`M${x - size * 0.3},${y + offset * size} 
                    Q${x - size * 0.15},${y + offset * size - 5} ${x},${y + offset * size}
                    Q${x + size * 0.15},${y + offset * size + 5} ${x + size * 0.3},${y + offset * size}`}
                fill="none"
                stroke="rgba(255,255,255,0.3)"
                strokeWidth="2"
                strokeLinecap="round"
              />
            ))}
          </g>
        );
      
      case "Desert":
        return (
          <g>
            <ellipse
              cx={x}
              cy={y + size * 0.1}
              rx={size * 0.35}
              ry={size * 0.15}
              fill="rgba(139,90,43,0.3)"
            />
            <ellipse
              cx={x - size * 0.2}
              cy={y - size * 0.15}
              rx={size * 0.2}
              ry={size * 0.1}
              fill="rgba(139,90,43,0.25)"
            />
            {[[-0.1, 0.3], [0.15, -0.1], [-0.25, 0], [0.2, 0.2], [0, -0.25]].map(([ox, oy], i) => (
              <circle
                key={i}
                cx={x + ox * size}
                cy={y + oy * size}
                r="1.5"
                fill="rgba(139,90,43,0.4)"
              />
            ))}
          </g>
        );
      
      default:
        return null;
    }
  };

  // For unexplored tiles, render as black
  if (visibilityState === "unexplored") {
    return (
      <g>
        <polygon
          points={points.join(" ")}
          fill="#0a0a0a"
          stroke="#000"
          strokeWidth="1"
        />
      </g>
    );
  }

  // For explored (fog) tiles, render dimmed terrain without decorations
  const opacity = visibilityState === "explored" ? 0.4 : 1;

  return (
    <g onClick={onClick} style={{ cursor: onClick ? "pointer" : "default", opacity }}>
      <defs>
        <linearGradient id={gradientId} x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor={color} />
          <stop offset="100%" stopColor={darkColor} />
        </linearGradient>
      </defs>
      
      <polygon
        points={points.join(" ")}
        fill={`url(#${gradientId})`}
        stroke={isSelected ? "#FFD700" : isHighlighted ? "#00FF00" : "#1a1a1a"}
        strokeWidth={isSelected ? "3" : isHighlighted ? "2.5" : "1.5"}
        className="transition-all hover:brightness-110"
      />
      
      {visibilityState === "visible" && renderTerrainDecoration()}
      
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
