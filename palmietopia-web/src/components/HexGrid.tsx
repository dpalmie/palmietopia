"use client";
import { useState, useRef } from "react";
import { HexTile } from "./HexTile";
import { City } from "./City";
import { Unit } from "./Unit";
import { City as CityType, Unit as UnitType, Player } from "@/hooks/useWebSocket";

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
  cities?: CityType[];
  units?: UnitType[];
  players?: Player[];
  selectedUnitId?: string | null;
  selectedCityId?: string | null;
  highlightedTiles?: { q: number; r: number }[];
  visibleTiles?: Set<string>;  // "q,r" strings for visible tiles
  exploredTiles?: Set<string>; // "q,r" strings for explored (but not visible) tiles
  onTileClick?: (q: number, r: number) => void;
  onUnitClick?: (unitId: string) => void;
  onCityClick?: (cityId: string) => void;
}

const PLAYER_COLORS: Record<string, string> = {
  Red: "#E53935",
  Blue: "#1E88E5",
  Green: "#43A047",
  Yellow: "#FDD835",
  Purple: "#8E24AA",
};

export function HexGrid({ 
  map, 
  hexSize = 30, 
  cities = [], 
  units = [], 
  players = [],
  selectedUnitId,
  selectedCityId,
  highlightedTiles = [],
  visibleTiles,
  exploredTiles,
  onTileClick,
  onUnitClick,
  onCityClick,
}: HexGridProps) {
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  const width = hexSize * Math.sqrt(3) * (map.radius * 2 + 1);
  const height = hexSize * 3 * map.radius + hexSize * 2;
  const centerX = width / 2;
  const centerY = height / 2;

  const getPlayerColor = (ownerId: string): string => {
    const player = players.find(p => p.id === ownerId);
    if (player) {
      return PLAYER_COLORS[player.color] || "#888";
    }
    return "#888";
  };

  const isHighlighted = (q: number, r: number): boolean => {
    return highlightedTiles.some(t => t.q === q && t.r === r);
  };

  // Determine visibility state: "visible" | "explored" | "unexplored"
  const getVisibilityState = (q: number, r: number): "visible" | "explored" | "unexplored" => {
    const key = `${q},${r}`;
    if (visibleTiles?.has(key)) return "visible";
    if (exploredTiles?.has(key)) return "explored";
    // If no visibility data provided, treat as visible (backwards compatibility)
    if (!visibleTiles && !exploredTiles) return "visible";
    return "unexplored";
  };

  // Check if a unit should be visible
  const isUnitVisible = (unit: UnitType): boolean => {
    if (!visibleTiles) return true; // No fog if no visibility data
    return visibleTiles.has(`${unit.q},${unit.r}`);
  };

  // Check if a city should be visible (cities are visible once explored)
  const isCityVisible = (city: CityType): boolean => {
    if (!visibleTiles && !exploredTiles) return true; // No fog
    const key = `${city.q},${city.r}`;
    return visibleTiles?.has(key) || exploredTiles?.has(key) || false;
  };

  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setZoom((z) => Math.min(Math.max(z * delta, 0.5), 3));
  };

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
          {/* Render tiles */}
          {map.tiles.map((tile) => (
            <HexTile
              key={`tile-${tile.q},${tile.r}`}
              q={tile.q}
              r={tile.r}
              terrain={tile.terrain}
              size={hexSize}
              onClick={onTileClick ? () => onTileClick(tile.q, tile.r) : undefined}
              isHighlighted={isHighlighted(tile.q, tile.r)}
              isSelected={false}
              visibilityState={getVisibilityState(tile.q, tile.r)}
            />
          ))}
          
          {/* Render cities (only if explored) */}
          {cities.filter(isCityVisible).map((city) => (
            <City
              key={`city-${city.id}`}
              q={city.q}
              r={city.r}
              size={hexSize}
              ownerColor={getPlayerColor(city.owner_id)}
              name={city.name}
              isSelected={city.id === selectedCityId}
              onClick={onCityClick ? () => onCityClick(city.id) : undefined}
              dimmed={!visibleTiles?.has(`${city.q},${city.r}`)}
            />
          ))}
          
          {/* Render units (only if visible) */}
          {units.filter(isUnitVisible).map((unit) => (
            <Unit
              key={`unit-${unit.id}`}
              q={unit.q}
              r={unit.r}
              size={hexSize}
              ownerColor={getPlayerColor(unit.owner_id)}
              unitType={unit.unit_type}
              isSelected={unit.id === selectedUnitId}
              onClick={onUnitClick ? () => onUnitClick(unit.id) : undefined}
              movementRemaining={unit.movement_remaining}
              hp={unit.hp ?? 100}
              maxHp={unit.max_hp ?? 100}
            />
          ))}
        </svg>
      </div>
    </div>
  );
}
