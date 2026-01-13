export type MapSize = "Tiny" | "Small" | "Medium" | "Large" | "Huge";

export interface Player {
  id: string;
  name: string;
  color: string;
}

export interface Lobby {
  id: string;
  host_id: string;
  players: Player[];
  map_size: MapSize;
  max_players: number;
  status: string;
}

export interface Tile {
  q: number;
  r: number;
  terrain: string;
}

export interface GameMap {
  tiles: Tile[];
  radius: number;
}

export interface GameSession {
  id: string;
  map: GameMap;
  players: Player[];
  current_turn: number;
  status: string;
}

export const MAP_SIZE_INFO: Record<MapSize, { radius: number; tiles: number; description: string }> = {
  Tiny: { radius: 2, tiles: 19, description: "Quick games, close combat" },
  Small: { radius: 4, tiles: 61, description: "Fast-paced strategy" },
  Medium: { radius: 6, tiles: 127, description: "Balanced gameplay" },
  Large: { radius: 8, tiles: 217, description: "Epic conflicts" },
  Huge: { radius: 10, tiles: 331, description: "Maximum strategic depth" },
};

export const PLAYER_COLORS: Record<string, string> = {
  Red: "#EF4444",
  Blue: "#3B82F6",
  Green: "#22C55E",
  Yellow: "#EAB308",
  Purple: "#A855F7",
};
