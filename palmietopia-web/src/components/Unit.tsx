interface UnitProps {
  q: number;
  r: number;
  size: number;
  ownerColor: string;
  unitType: string;
  isSelected?: boolean;
  onClick?: () => void;
  movementRemaining: number;
}

export function Unit({ q, r, size, ownerColor, unitType, isSelected, onClick, movementRemaining }: UnitProps) {
  const x = size * (Math.sqrt(3) * q + (Math.sqrt(3) / 2) * r);
  const y = size * ((3 / 2) * r);
  
  const s = size * 0.35;

  const renderConscript = () => (
    <g>
      {/* Body/shield */}
      <ellipse
        cx={x}
        cy={y + s * 0.2}
        rx={s * 0.6}
        ry={s * 0.8}
        fill="#8B4513"
        stroke={ownerColor}
        strokeWidth={isSelected ? "3" : "2"}
      />
      
      {/* Head */}
      <circle
        cx={x}
        cy={y - s * 0.5}
        r={s * 0.35}
        fill="#D2B48C"
        stroke={ownerColor}
        strokeWidth="1.5"
      />
      
      {/* Helmet */}
      <ellipse
        cx={x}
        cy={y - s * 0.65}
        rx={s * 0.38}
        ry={s * 0.25}
        fill="#4a4a4a"
      />
      
      {/* Spear */}
      <line
        x1={x + s * 0.5}
        y1={y + s * 0.8}
        x2={x + s * 0.3}
        y2={y - s * 1.2}
        stroke="#5D4037"
        strokeWidth="3"
      />
      {/* Spear tip */}
      <polygon
        points={`${x + s * 0.3},${y - s * 1.2} 
                 ${x + s * 0.2},${y - s * 1.5} 
                 ${x + s * 0.4},${y - s * 1.5}`}
        fill="#9E9E9E"
      />
      
      {/* Shield emblem (owner color) */}
      <circle
        cx={x}
        cy={y + s * 0.2}
        r={s * 0.25}
        fill={ownerColor}
        opacity="0.8"
      />
    </g>
  );

  return (
    <g 
      onClick={onClick} 
      style={{ cursor: onClick ? "pointer" : "default" }}
      className={isSelected ? "animate-pulse" : ""}
    >
      {/* Selection ring */}
      {isSelected && (
        <circle
          cx={x}
          cy={y}
          r={s * 1.5}
          fill="none"
          stroke="#FFD700"
          strokeWidth="2"
          strokeDasharray="5,3"
        />
      )}
      
      {unitType === "Conscript" && renderConscript()}
      
      {/* Movement indicator */}
      <g transform={`translate(${x + s * 0.8}, ${y + s * 0.8})`}>
        <circle r={s * 0.35} fill="#1a1a1a" opacity="0.8" />
        <text
          textAnchor="middle"
          dominantBaseline="central"
          fill={movementRemaining > 0 ? "#4CAF50" : "#F44336"}
          fontSize={s * 0.4}
          fontWeight="bold"
        >
          {movementRemaining}
        </text>
      </g>
    </g>
  );
}
