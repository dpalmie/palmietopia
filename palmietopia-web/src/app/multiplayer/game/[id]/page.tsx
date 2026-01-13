"use client";
import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { HexGrid } from "@/components/HexGrid";
import { useWebSocket, GameSession } from "@/hooks/useWebSocket";
import { PLAYER_COLORS } from "@/types/game";

function formatTime(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export default function GamePage() {
  const params = useParams();
  const router = useRouter();
  const gameId = params.id as string;

  const {
    isConnected,
    playerId,
    game,
    turnTimeRemaining,
    endTurn,
    rejoinGame,
  } = useWebSocket();

  const [initialGame, setInitialGame] = useState<GameSession | null>(null);
  const [localTimeRemaining, setLocalTimeRemaining] = useState<number>(0);
  const [myPlayerId, setMyPlayerId] = useState<string | null>(null);

  // Load initial game and player ID from sessionStorage
  useEffect(() => {
    const storedGame = sessionStorage.getItem(`game-${gameId}`);
    const storedPlayerId = sessionStorage.getItem(`player-${gameId}`);
    
    if (storedGame) {
      try {
        setInitialGame(JSON.parse(storedGame));
      } catch {
        // Invalid stored game
      }
    }
    
    if (storedPlayerId) {
      setMyPlayerId(storedPlayerId);
    }
  }, [gameId]);

  // Rejoin game when connected and we have player ID
  useEffect(() => {
    if (isConnected && myPlayerId && gameId) {
      console.log("Rejoining game:", { gameId, myPlayerId });
      rejoinGame(gameId, myPlayerId);
    }
  }, [isConnected, myPlayerId, gameId, rejoinGame]);

  // Use live game state if available, otherwise use initial
  const currentGame = game || initialGame;

  // Initialize localTimeRemaining when game loads (per-player time)
  useEffect(() => {
    if (currentGame) {
      const currentPlayerTime = currentGame.player_times_ms[currentGame.current_turn];
      setLocalTimeRemaining(currentPlayerTime);
    }
  }, [currentGame]);

  // Client-side countdown timer based on turn_started_at_ms
  useEffect(() => {
    if (!currentGame) return;
    
    const updateTimer = () => {
      const now = Date.now();
      // Fallback: if turn_started_at_ms is 0/missing, assume turn just started
      const startTime = currentGame.turn_started_at_ms > 0 
        ? currentGame.turn_started_at_ms 
        : now;
      const elapsed = now - startTime;
      // Use current player's time bank
      const currentPlayerTime = currentGame.player_times_ms[currentGame.current_turn];
      const remaining = Math.max(0, currentPlayerTime - elapsed);
      setLocalTimeRemaining(remaining);
    };

    updateTimer();
    const interval = setInterval(updateTimer, 100);
    return () => clearInterval(interval);
  }, [currentGame?.turn_started_at_ms, currentGame?.current_turn, currentGame]);

  const handleLeaveGame = () => {
    sessionStorage.removeItem(`game-${gameId}`);
    router.push("/multiplayer");
  };

  const handleEndTurn = () => {
    if (myPlayerId) {
      console.log("Ending turn:", { gameId, myPlayerId });
      endTurn(gameId, myPlayerId);
    } else {
      console.error("Cannot end turn: myPlayerId is null");
    }
  };

  if (!isConnected) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-zinc-900">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-emerald-500"></div>
        <p className="text-zinc-400 mt-4">Connecting...</p>
      </div>
    );
  }

  if (!currentGame) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-zinc-900">
        <div className="text-center">
          <p className="text-red-400 mb-4">Game session not found</p>
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

  const currentPlayer = currentGame.players[currentGame.current_turn];
  // Use stored player ID (from when we joined the lobby) instead of new WebSocket's player ID
  const isMyTurn = currentPlayer?.id === myPlayerId;
  // Use server time if available, otherwise local calculation
  const timeRemaining = turnTimeRemaining || localTimeRemaining;
  const isLowTime = timeRemaining < 30000;

  return (
    <div className="flex h-screen flex-col bg-zinc-900">
      <header className="p-4 bg-zinc-800 border-b border-zinc-700 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-zinc-50">Palmietopia</h1>
          <p className="text-sm text-zinc-400">
            {currentGame.map.tiles.length} tiles • {currentGame.players.length} players
          </p>
        </div>
        <div className="flex items-center gap-4">
          {/* Timer */}
          <div className={`text-2xl font-mono font-bold ${isLowTime ? "text-red-500" : "text-zinc-50"}`}>
            {formatTime(timeRemaining)}
          </div>
          
          {/* End Turn Button */}
          <button
            onClick={handleEndTurn}
            disabled={!isMyTurn}
            className={`px-4 py-2 rounded font-medium transition-colors ${
              isMyTurn
                ? "bg-emerald-600 hover:bg-emerald-500 text-white"
                : "bg-zinc-700 text-zinc-500 cursor-not-allowed"
            }`}
          >
            End Turn
          </button>

          <button
            onClick={handleLeaveGame}
            className="px-4 py-2 bg-red-600 hover:bg-red-500 rounded text-white transition-colors"
          >
            Leave
          </button>
        </div>
      </header>

      {/* Player bar */}
      <div className="px-4 py-2 bg-zinc-800/50 border-b border-zinc-700 flex items-center gap-4">
        {currentGame.players.map((player, index) => {
          const isCurrentTurn = index === currentGame.current_turn;
          const isMe = player.id === myPlayerId;
          
          return (
            <div
              key={player.id}
              className={`flex items-center gap-2 px-3 py-1.5 rounded transition-colors ${
                isCurrentTurn ? "bg-zinc-700 ring-2 ring-emerald-500" : ""
              }`}
            >
              <div
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: PLAYER_COLORS[player.color] || "#888" }}
              />
              <span className={`text-sm ${isCurrentTurn ? "text-zinc-50 font-medium" : "text-zinc-400"}`}>
                {player.name}
                {isMe && " (You)"}
              </span>
              {isCurrentTurn && (
                <span className="text-xs text-emerald-400 ml-1">●</span>
              )}
            </div>
          );
        })}
      </div>

      <main className="flex-1 overflow-hidden">
        <HexGrid map={currentGame.map} hexSize={40} />
      </main>
    </div>
  );
}
