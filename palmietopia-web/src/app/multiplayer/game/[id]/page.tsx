"use client";
import { useEffect, useState, useCallback } from "react";
import { useParams, useRouter } from "next/navigation";
import { HexGrid } from "@/components/HexGrid";
import { GameOverDialog } from "@/components/GameOverDialog";
import { useWebSocket, GameSession, Unit } from "@/hooks/useWebSocket";
import { PLAYER_COLORS } from "@/types/game";

function formatTime(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function hexDistance(q1: number, r1: number, q2: number, r2: number): number {
  return (Math.abs(q1 - q2) + Math.abs(r1 - r2) + Math.abs(q1 + r1 - q2 - r2)) / 2;
}

function getAdjacentTiles(q: number, r: number): { q: number; r: number }[] {
  return [
    { q: q + 1, r },
    { q: q - 1, r },
    { q, r: r + 1 },
    { q, r: r - 1 },
    { q: q + 1, r: r - 1 },
    { q: q - 1, r: r + 1 },
  ];
}

export default function GamePage() {
  const params = useParams();
  const router = useRouter();
  const gameId = params.id as string;

  const {
    isConnected,
    game,
    turnTimeRemaining,
    endTurn,
    rejoinGame,
    moveUnit,
    attackUnit,
  } = useWebSocket();

  const [initialGame, setInitialGame] = useState<GameSession | null>(null);
  const [localTimeRemaining, setLocalTimeRemaining] = useState<number>(0);
  const [myPlayerId, setMyPlayerId] = useState<string | null>(null);
  const [selectedUnitId, setSelectedUnitId] = useState<string | null>(null);
  const [highlightedTiles, setHighlightedTiles] = useState<{ q: number; r: number }[]>([]);

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

  useEffect(() => {
    if (isConnected && myPlayerId && gameId) {
      console.log("Rejoining game:", { gameId, myPlayerId });
      rejoinGame(gameId, myPlayerId);
    }
  }, [isConnected, myPlayerId, gameId, rejoinGame]);

  const currentGame = game || initialGame;

  useEffect(() => {
    if (currentGame) {
      const currentPlayerTime = currentGame.player_times_ms[currentGame.current_turn];
      setLocalTimeRemaining(currentPlayerTime);
    }
  }, [currentGame]);

  useEffect(() => {
    if (!currentGame) return;
    
    const updateTimer = () => {
      const now = Date.now();
      const startTime = currentGame.turn_started_at_ms > 0 
        ? currentGame.turn_started_at_ms 
        : now;
      const elapsed = now - startTime;
      const currentPlayerTime = currentGame.player_times_ms[currentGame.current_turn];
      const remaining = Math.max(0, currentPlayerTime - elapsed);
      setLocalTimeRemaining(remaining);
    };

    updateTimer();
    const interval = setInterval(updateTimer, 100);
    return () => clearInterval(interval);
  }, [currentGame?.turn_started_at_ms, currentGame?.current_turn, currentGame]);

  // Calculate valid movement tiles for selected unit
  const calculateMovementTiles = useCallback((unit: Unit) => {
    if (!currentGame) return [];
    
    const validTiles: { q: number; r: number }[] = [];
    const adjacent = getAdjacentTiles(unit.q, unit.r);
    
    for (const pos of adjacent) {
      const tile = currentGame.map.tiles.find(t => t.q === pos.q && t.r === pos.r);
      if (!tile) continue;
      
      // Check terrain passability and cost
      let cost = 0;
      if (tile.terrain === "Water") continue;
      if (tile.terrain === "Mountain") cost = 2;
      else cost = 1;
      
      if (unit.movement_remaining >= cost) {
        // Check if tile is occupied by another unit
        const occupied = (currentGame.units || []).some(u => u.q === pos.q && u.r === pos.r);
        if (!occupied) {
          validTiles.push(pos);
        }
      }
    }
    
    return validTiles;
  }, [currentGame]);

  const handleUnitClick = useCallback((unitId: string) => {
    if (!currentGame || !myPlayerId) return;
    
    const clickedUnit = (currentGame.units || []).find(u => u.id === unitId);
    if (!clickedUnit) return;
    
    const currentPlayer = currentGame.players[currentGame.current_turn];
    const isMyTurn = currentPlayer.id === myPlayerId;
    
    // If we have a unit selected and click an enemy unit, try to attack
    if (selectedUnitId && clickedUnit.owner_id !== myPlayerId && isMyTurn) {
      const myUnit = (currentGame.units || []).find(u => u.id === selectedUnitId);
      if (myUnit && myUnit.movement_remaining > 0) {
        // Check if adjacent
        const distance = hexDistance(myUnit.q, myUnit.r, clickedUnit.q, clickedUnit.r);
        if (distance === 1) {
          attackUnit(gameId, myPlayerId, selectedUnitId, unitId);
          setSelectedUnitId(null);
          setHighlightedTiles([]);
          return;
        }
      }
    }
    
    // Can only select own units on your turn
    if (!isMyTurn || clickedUnit.owner_id !== myPlayerId) {
      setSelectedUnitId(null);
      setHighlightedTiles([]);
      return;
    }
    
    if (selectedUnitId === unitId) {
      // Deselect
      setSelectedUnitId(null);
      setHighlightedTiles([]);
    } else {
      // Select and show movement options
      setSelectedUnitId(unitId);
      setHighlightedTiles(calculateMovementTiles(clickedUnit));
    }
  }, [currentGame, myPlayerId, selectedUnitId, calculateMovementTiles, gameId, attackUnit]);

  const handleTileClick = useCallback((q: number, r: number) => {
    if (!selectedUnitId || !myPlayerId || !currentGame) return;
    
    // Check if this tile is in highlighted (valid move) tiles
    const isValidMove = highlightedTiles.some(t => t.q === q && t.r === r);
    if (!isValidMove) {
      // Deselect if clicking invalid tile
      setSelectedUnitId(null);
      setHighlightedTiles([]);
      return;
    }
    
    // Move the unit
    moveUnit(gameId, myPlayerId, selectedUnitId, q, r);
    setSelectedUnitId(null);
    setHighlightedTiles([]);
  }, [selectedUnitId, myPlayerId, currentGame, highlightedTiles, gameId, moveUnit]);

  // Update highlighted tiles when game state changes
  useEffect(() => {
    if (selectedUnitId && currentGame) {
      const unit = (currentGame.units || []).find(u => u.id === selectedUnitId);
      if (unit) {
        setHighlightedTiles(calculateMovementTiles(unit));
      } else {
        setSelectedUnitId(null);
        setHighlightedTiles([]);
      }
    }
  }, [currentGame?.units, selectedUnitId, calculateMovementTiles]);

  const handleLeaveGame = () => {
    sessionStorage.removeItem(`game-${gameId}`);
    router.push("/multiplayer");
  };

  const handleEndTurn = () => {
    if (myPlayerId) {
      console.log("Ending turn:", { gameId, myPlayerId });
      endTurn(gameId, myPlayerId);
      setSelectedUnitId(null);
      setHighlightedTiles([]);
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
  const isMyTurn = currentPlayer?.id === myPlayerId;
  const timeRemaining = turnTimeRemaining || localTimeRemaining;
  const isLowTime = timeRemaining < 30000;
  const isEliminated = (currentGame.eliminated_players || []).includes(myPlayerId || "");
  const isVictory = typeof currentGame.status === "object" && "Victory" in currentGame.status;
  const winnerId = isVictory ? (currentGame.status as { Victory: { winner_id: string } }).Victory.winner_id : null;
  const winnerName = winnerId ? currentGame.players.find(p => p.id === winnerId)?.name || null : null;

  return (
    <div className="flex h-screen flex-col bg-zinc-900">
      <header className="p-4 bg-zinc-800 border-b border-zinc-700 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-zinc-50">Palmietopia</h1>
          <p className="text-sm text-zinc-400">
            {currentGame.map.tiles.length} tiles • {currentGame.players.length} players • {currentGame.cities?.length || 0} cities • {currentGame.units?.length || 0} units
          </p>
        </div>
        <div className="flex items-center gap-4">
          <div className={`text-2xl font-mono font-bold ${isLowTime ? "text-red-500" : "text-zinc-50"}`}>
            {formatTime(timeRemaining)}
          </div>
          
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

      <div className="px-4 py-2 bg-zinc-800/50 border-b border-zinc-700 flex items-center gap-4">
        {currentGame.players.map((player, index) => {
          const isCurrentTurn = index === currentGame.current_turn;
          const isMe = player.id === myPlayerId;
          const isPlayerEliminated = (currentGame.eliminated_players || []).includes(player.id);
          
          return (
            <div
              key={player.id}
              className={`flex items-center gap-2 px-3 py-1.5 rounded transition-colors ${
                isPlayerEliminated ? "opacity-40" : ""
              } ${
                isCurrentTurn && !isPlayerEliminated ? "bg-zinc-700 ring-2 ring-emerald-500" : ""
              }`}
            >
              <div
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: PLAYER_COLORS[player.color] || "#888" }}
              />
              <span className={`text-sm ${isPlayerEliminated ? "line-through text-zinc-500" : isCurrentTurn ? "text-zinc-50 font-medium" : "text-zinc-400"}`}>
                {player.name}
                {isMe && " (You)"}
                {isPlayerEliminated && " [Eliminated]"}
              </span>
              {isCurrentTurn && !isPlayerEliminated && (
                <span className="text-xs text-emerald-400 ml-1">●</span>
              )}
            </div>
          );
        })}
      </div>

      {/* Instructions */}
      {isMyTurn && !isEliminated && (
        <div className="px-4 py-2 bg-emerald-900/30 text-emerald-300 text-sm text-center">
          Your turn! Click a unit to select it, then click a tile to move or an enemy unit to attack.
        </div>
      )}
      {isEliminated && (
        <div className="px-4 py-2 bg-red-900/30 text-red-300 text-sm text-center">
          You have been eliminated! Your capitol was captured.
        </div>
      )}

      <main className="flex-1 overflow-hidden">
        <HexGrid 
          map={currentGame.map} 
          hexSize={40}
          cities={currentGame.cities || []}
          units={currentGame.units || []}
          players={currentGame.players}
          selectedUnitId={selectedUnitId}
          highlightedTiles={highlightedTiles}
          onTileClick={handleTileClick}
          onUnitClick={handleUnitClick}
        />
      </main>

      {isVictory && (
        <GameOverDialog
          isWinner={winnerId === myPlayerId}
          winnerName={winnerName}
          onClose={() => router.push("/multiplayer")}
        />
      )}
    </div>
  );
}
