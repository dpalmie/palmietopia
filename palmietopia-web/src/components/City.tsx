interface CityProps {
  q: number;
  r: number;
  size: number;
  ownerColor: string;
  name: string;
}

export function City({ q, r, size, ownerColor, name }: CityProps) {
  const x = size * (Math.sqrt(3) * q + (Math.sqrt(3) / 2) * r);
  const y = size * ((3 / 2) * r);
  
  const s = size * 0.4;

  return (
    <g className="pointer-events-none">
      {/* Castle base */}
      <rect
        x={x - s * 0.8}
        y={y - s * 0.3}
        width={s * 1.6}
        height={s * 0.9}
        fill="#4a4a4a"
        stroke={ownerColor}
        strokeWidth="2"
      />
      
      {/* Left tower */}
      <rect
        x={x - s * 0.9}
        y={y - s * 0.8}
        width={s * 0.4}
        height={s * 1.4}
        fill="#5a5a5a"
        stroke={ownerColor}
        strokeWidth="1.5"
      />
      {/* Left tower crenellation */}
      <rect x={x - s * 0.9} y={y - s * 0.95} width={s * 0.15} height={s * 0.2} fill="#5a5a5a" />
      <rect x={x - s * 0.65} y={y - s * 0.95} width={s * 0.15} height={s * 0.2} fill="#5a5a5a" />
      
      {/* Right tower */}
      <rect
        x={x + s * 0.5}
        y={y - s * 0.8}
        width={s * 0.4}
        height={s * 1.4}
        fill="#5a5a5a"
        stroke={ownerColor}
        strokeWidth="1.5"
      />
      {/* Right tower crenellation */}
      <rect x={x + s * 0.5} y={y - s * 0.95} width={s * 0.15} height={s * 0.2} fill="#5a5a5a" />
      <rect x={x + s * 0.75} y={y - s * 0.95} width={s * 0.15} height={s * 0.2} fill="#5a5a5a" />
      
      {/* Center tower (taller) */}
      <rect
        x={x - s * 0.25}
        y={y - s * 1.1}
        width={s * 0.5}
        height={s * 1.7}
        fill="#6a6a6a"
        stroke={ownerColor}
        strokeWidth="1.5"
      />
      
      {/* Flag on center tower */}
      <line
        x1={x}
        y1={y - s * 1.1}
        x2={x}
        y2={y - s * 1.5}
        stroke="#333"
        strokeWidth="2"
      />
      <polygon
        points={`${x},${y - s * 1.5} ${x + s * 0.3},${y - s * 1.35} ${x},${y - s * 1.2}`}
        fill={ownerColor}
      />
      
      {/* Door */}
      <rect
        x={x - s * 0.15}
        y={y + s * 0.1}
        width={s * 0.3}
        height={s * 0.5}
        fill="#2a2a2a"
        rx="2"
      />
      
      {/* Windows */}
      <rect x={x - s * 0.55} y={y - s * 0.1} width={s * 0.15} height={s * 0.2} fill="#2a2a2a" />
      <rect x={x + s * 0.4} y={y - s * 0.1} width={s * 0.15} height={s * 0.2} fill="#2a2a2a" />
    </g>
  );
}
