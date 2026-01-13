"use client";
import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { HexGrid } from "@/components/HexGrid";
import { GameSession, PLAYER_COLORS } from "@/types/game";

export default function GamePage() {
  const params = useParams();
  const router = useRouter();
  const gameId = params.id as string;

  const [game, setGame] = useState<GameSession | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Try to get game state from sessionStorage (set by lobby page)
    const storedGame = sessionStorage.getItem(`game-${gameId}`);
    if (storedGame) {
      try {
        setGame(JSON.parse(storedGame));
      } catch {
        setError("Failed to load game state");
      }
    } else {
      // For now, if no stored game, show error
      // In a full implementation, we'd reconnect via WebSocket
      setError("Game session not found. Please rejoin from the lobby.");
    }
  }, [gameId]);

  const handleLeaveGame = () => {
    sessionStorage.removeItem(`game-${gameId}`);
    router.push("/multiplayer");
  };

  if (error) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-zinc-900">
        <div className="text-center">
          <p className="text-red-400 mb-4">{error}</p>
          <button
            onClick={() => router.push("/multiplayer")}
            className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 rounded text-zinc-50 transition-colors"
          >
            Back to Multiplayer
          </button>
        </div>
      </div>
    );
  }

  if (!game) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-zinc-900">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-emerald-500"></div>
        <p className="text-zinc-400 mt-4">Loading game...</p>
      </div>
    );
  }

  const currentPlayer = game.players[game.current_turn];

  return (
    <div className="flex h-screen flex-col bg-zinc-900">
      <header className="p-4 bg-zinc-800 border-b border-zinc-700 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-zinc-50">Palmietopia</h1>
          <p className="text-sm text-zinc-400">
            {game.map.tiles.length} tiles • {game.players.length} players
          </p>
        </div>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <span className="text-zinc-400 text-sm">Turn:</span>
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: PLAYER_COLORS[currentPlayer?.color] || "#888" }}
            />
            <span className="text-zinc-50 font-medium">{currentPlayer?.name}</span>
          </div>
          <button
            onClick={handleLeaveGame}
            className="px-4 py-2 bg-red-600 hover:bg-red-500 rounded text-white transition-colors"
          >
            Leave Game
          </button>
        </div>
      </header>

      {/* Player bar */}
      <div className="px-4 py-2 bg-zinc-800/50 border-b border-zinc-700 flex items-center gap-4">
        {game.players.map((player, index) => (
          <div
            key={player.id}
            className={`flex items-center gap-2 px-3 py-1 rounded ${
              index === game.current_turn ? "bg-zinc-700" : ""
            }`}
          >
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: PLAYER_COLORS[player.color] || "#888" }}
            />
            <span className={`text-sm ${index === game.current_turn ? "text-zinc-50 font-medium" : "text-zinc-400"}`}>
              {player.name}
            </span>
          </div>
        ))}
      </div>

      <main className="flex-1 overflow-hidden">
        <HexGrid map={game.map} hexSize={40} />
      </main>
    </div>
  );
}
