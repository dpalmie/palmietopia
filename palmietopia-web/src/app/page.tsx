"use client";
import { useEffect, useState } from "react";
import { HexGrid } from "@/components/HexGrid";

interface Tile {
  q: number;
  r: number;
  terrain: string;
}

interface GameMap {
  tiles: Tile[];
  radius: number;
}

export default function Home() {
  const [map, setMap] = useState<GameMap | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadWasm() {
      try {
        const wasm = await import("../../pkg/palmietopia_core");
        await wasm.default("/wasm/palmietopia_core_bg.wasm");

        // Generate and parse the tiny map
        const mapJson = wasm.generate_tiny_map();
        const gameMap = JSON.parse(mapJson);
        setMap(gameMap);
      } catch (error) {
        console.error("Failed to load WASM module:", error);
        setError("Error loading WASM module");
      }
    }
    loadWasm();
  }, []);

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-zinc-900">
        <p className="text-red-500">{error}</p>
      </div>
    );
  }

  if (!map) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-zinc-900">
        <p className="text-zinc-400">Loading map...</p>
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col bg-zinc-900">
      <header className="p-4 bg-zinc-800 border-b border-zinc-700">
        <h1 className="text-2xl font-bold text-zinc-50">Palmietopia</h1>
        <p className="text-sm text-zinc-400">Tiny Map - {map.tiles.length} tiles</p>
      </header>
      <main className="flex-1 overflow-hidden">
        <HexGrid map={map} hexSize={40} />
      </main>
    </div>
  );
}
