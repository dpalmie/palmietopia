"use client";
import { useState, useRef } from "react";
import { HexTile } from "./HexTile";

interface Tile {
  q: number;
  r: number;
  terrain: string;
}

interface GameMap {
  tiles: Tile[];
  radius: number;
}

interface HexGridProps {
  map: GameMap;
  hexSize?: number;
}

export function HexGrid({ map, hexSize = 30 }: HexGridProps) {
  // Pan/zoom state
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate viewBox dimensions based on map radius
  const width = hexSize * Math.sqrt(3) * (map.radius * 2 + 1);
  const height = hexSize * 3 * map.radius + hexSize * 2;

  // Center the map
  const centerX = width / 2;
  const centerY = height / 2;

  // Mouse wheel zoom handler
  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setZoom((z) => Math.min(Math.max(z * delta, 0.5), 3));
  };

  // Mouse drag pan handlers
  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging) return;
    setPan({
      x: e.clientX - dragStart.x,
      y: e.clientY - dragStart.y,
    });
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  const handleMouseLeave = () => {
    setIsDragging(false);
  };

  return (
    <div
      ref={containerRef}
      className="w-full h-full overflow-hidden cursor-grab active:cursor-grabbing"
      onWheel={handleWheel}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseLeave}
    >
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          transform: `
            perspective(1000px)
            rotateX(55deg)
            scale(${zoom})
            translate(${pan.x}px, ${pan.y}px)
          `,
          transformStyle: "preserve-3d",
          transition: isDragging ? "none" : "transform 0.1s ease-out",
        }}
      >
        <svg
          width={width}
          height={height}
          viewBox={`${-centerX} ${-centerY} ${width} ${height}`}
          style={{ display: "block" }}
        >
          {map.tiles.map((tile) => (
            <HexTile
              key={`${tile.q},${tile.r}`}
              q={tile.q}
              r={tile.r}
              terrain={tile.terrain}
              size={hexSize}
            />
          ))}
        </svg>
      </div>
    </div>
  );
}
