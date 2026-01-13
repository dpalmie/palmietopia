"use client";
import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useWebSocket, MapSize } from "@/hooks/useWebSocket";
import { MAP_SIZE_INFO, PLAYER_COLORS } from "@/types/game";

export default function MultiplayerPage() {
  const router = useRouter();
  const {
    isConnected,
    playerId,
    currentLobby,
    lobbies,
    game,
    error,
    createLobby,
    joinLobby,
    leaveLobby,
    startGame,
    listLobbies,
    setError,
  } = useWebSocket();

  const [playerName, setPlayerName] = useState("");
  const [selectedMapSize, setSelectedMapSize] = useState<MapSize>("Small");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [joinLobbyId, setJoinLobbyId] = useState<string | null>(null);

  // Refresh lobby list periodically
  useEffect(() => {
    if (!isConnected) return;
    const interval = setInterval(listLobbies, 5000);
    return () => clearInterval(interval);
  }, [isConnected, listLobbies]);

  useEffect(() => {
    if (game) {
      router.push(`/multiplayer/game/${game.id}`);
    }
  }, [game, router]);

  const handleCreateLobby = (e: React.FormEvent) => {
    e.preventDefault();
    if (!playerName.trim()) {
      setError("Please enter your name");
      return;
    }
    createLobby(playerName.trim(), selectedMapSize);
    setShowCreateForm(false);
  };

  const handleJoinLobby = (e: React.FormEvent) => {
    e.preventDefault();
    if (!playerName.trim()) {
      setError("Please enter your name");
      return;
    }
    if (joinLobbyId) {
      joinLobby(joinLobbyId, playerName.trim());
      setJoinLobbyId(null);
    }
  };

  const handleStartGame = () => {
    startGame();
  };

  const handleLeaveLobby = () => {
    leaveLobby();
  };

  if (!isConnected) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-zinc-900">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-emerald-500 mx-auto mb-4"></div>
          <p className="text-zinc-400">Connecting to server...</p>
        </div>
      </div>
    );
  }

  // In a lobby - show lobby room
  if (currentLobby) {
    const isHost = currentLobby.host_id === playerId;

    return (
      <div className="flex min-h-screen flex-col bg-zinc-900">
        <header className="p-4 bg-zinc-800 border-b border-zinc-700 flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-zinc-50">Game Lobby</h1>
            <p className="text-sm text-zinc-400">
              {MAP_SIZE_INFO[currentLobby.map_size].description} • {currentLobby.players.length}/{currentLobby.max_players} players
            </p>
          </div>
          <button
            onClick={handleLeaveLobby}
            className="px-4 py-2 bg-red-600 hover:bg-red-500 rounded text-white transition-colors"
          >
            Leave Lobby
          </button>
        </header>

        <main className="flex-1 p-8">
          <div className="max-w-2xl mx-auto">
            <div className="bg-zinc-800 rounded-lg p-6 mb-6">
              <h2 className="text-xl font-semibold text-zinc-50 mb-4">Players</h2>
              <div className="space-y-3">
                {currentLobby.players.map((player, index) => (
                  <div
                    key={player.id}
                    className="flex items-center gap-3 p-3 bg-zinc-700 rounded-lg"
                  >
                    <div
                      className="w-4 h-4 rounded-full"
                      style={{ backgroundColor: PLAYER_COLORS[player.color] || "#888" }}
                    />
                    <span className="text-zinc-50 font-medium">{player.name}</span>
                    {player.id === currentLobby.host_id && (
                      <span className="text-xs bg-emerald-600 px-2 py-1 rounded text-white">Host</span>
                    )}
                  </div>
                ))}
                {Array.from({ length: currentLobby.max_players - currentLobby.players.length }).map((_, i) => (
                  <div
                    key={`empty-${i}`}
                    className="flex items-center gap-3 p-3 bg-zinc-700/50 rounded-lg border border-dashed border-zinc-600"
                  >
                    <div className="w-4 h-4 rounded-full bg-zinc-600" />
                    <span className="text-zinc-500">Waiting for player...</span>
                  </div>
                ))}
              </div>
            </div>

            <div className="bg-zinc-800 rounded-lg p-6 mb-6">
              <h2 className="text-xl font-semibold text-zinc-50 mb-2">Map Settings</h2>
              <div className="text-zinc-400">
                <p>Size: <span className="text-zinc-50">{currentLobby.map_size}</span></p>
                <p>Tiles: <span className="text-zinc-50">{MAP_SIZE_INFO[currentLobby.map_size].tiles}</span></p>
              </div>
            </div>

            {isHost ? (
              <button
                onClick={handleStartGame}
                disabled={currentLobby.players.length < 2}
                className="w-full px-6 py-4 bg-emerald-600 hover:bg-emerald-500 disabled:bg-zinc-600 disabled:cursor-not-allowed rounded-lg text-white text-xl font-semibold transition-colors"
              >
                {currentLobby.players.length < 2 ? "Waiting for players..." : "Start Game"}
              </button>
            ) : (
              <div className="text-center p-4 bg-zinc-800 rounded-lg">
                <p className="text-zinc-400">Waiting for host to start the game...</p>
              </div>
            )}
          </div>
        </main>
      </div>
    );
  }

  // Lobby browser
  return (
    <div className="flex min-h-screen flex-col bg-zinc-900">
      <header className="p-4 bg-zinc-800 border-b border-zinc-700 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-zinc-50">Multiplayer</h1>
          <p className="text-sm text-zinc-400">Join a game or create your own</p>
        </div>
        <Link
          href="/"
          className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 rounded text-zinc-50 transition-colors"
        >
          Back to Menu
        </Link>
      </header>

      <main className="flex-1 p-8">
        <div className="max-w-4xl mx-auto">
          {error && (
            <div className="mb-6 p-4 bg-red-900/50 border border-red-700 rounded-lg text-red-200">
              {error}
              <button onClick={() => setError(null)} className="ml-4 underline">Dismiss</button>
            </div>
          )}

          {/* Create Lobby Section */}
          <div className="mb-8">
            {!showCreateForm ? (
              <button
                onClick={() => setShowCreateForm(true)}
                className="w-full px-6 py-4 bg-emerald-700 hover:bg-emerald-600 border border-emerald-500 rounded-lg text-zinc-50 text-xl font-semibold transition-colors"
              >
                + Create New Game
              </button>
            ) : (
              <form onSubmit={handleCreateLobby} className="bg-zinc-800 rounded-lg p-6">
                <h2 className="text-xl font-semibold text-zinc-50 mb-4">Create New Game</h2>
                
                <div className="mb-4">
                  <label className="block text-zinc-400 mb-2">Your Name</label>
                  <input
                    type="text"
                    value={playerName}
                    onChange={(e) => setPlayerName(e.target.value)}
                    className="w-full px-4 py-2 bg-zinc-700 border border-zinc-600 rounded text-zinc-50 focus:outline-none focus:border-emerald-500"
                    placeholder="Enter your name"
                    maxLength={20}
                  />
                </div>

                <div className="mb-6">
                  <label className="block text-zinc-400 mb-2">Map Size</label>
                  <div className="grid grid-cols-5 gap-2">
                    {(Object.keys(MAP_SIZE_INFO) as MapSize[]).map((size) => (
                      <button
                        key={size}
                        type="button"
                        onClick={() => setSelectedMapSize(size)}
                        className={`px-3 py-2 rounded text-sm font-medium transition-colors ${
                          selectedMapSize === size
                            ? "bg-emerald-600 text-white"
                            : "bg-zinc-700 text-zinc-300 hover:bg-zinc-600"
                        }`}
                      >
                        {size}
                      </button>
                    ))}
                  </div>
                  <p className="mt-2 text-sm text-zinc-500">
                    {MAP_SIZE_INFO[selectedMapSize].tiles} tiles • {MAP_SIZE_INFO[selectedMapSize].description}
                  </p>
                </div>

                <div className="flex gap-3">
                  <button
                    type="submit"
                    className="flex-1 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 rounded text-white font-medium transition-colors"
                  >
                    Create Lobby
                  </button>
                  <button
                    type="button"
                    onClick={() => setShowCreateForm(false)}
                    className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 rounded text-zinc-300 transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </form>
            )}
          </div>

          {/* Join Lobby Modal */}
          {joinLobbyId && (
            <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
              <form onSubmit={handleJoinLobby} className="bg-zinc-800 rounded-lg p-6 w-96">
                <h2 className="text-xl font-semibold text-zinc-50 mb-4">Join Game</h2>
                <div className="mb-4">
                  <label className="block text-zinc-400 mb-2">Your Name</label>
                  <input
                    type="text"
                    value={playerName}
                    onChange={(e) => setPlayerName(e.target.value)}
                    className="w-full px-4 py-2 bg-zinc-700 border border-zinc-600 rounded text-zinc-50 focus:outline-none focus:border-emerald-500"
                    placeholder="Enter your name"
                    maxLength={20}
                    autoFocus
                  />
                </div>
                <div className="flex gap-3">
                  <button
                    type="submit"
                    className="flex-1 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 rounded text-white font-medium transition-colors"
                  >
                    Join
                  </button>
                  <button
                    type="button"
                    onClick={() => setJoinLobbyId(null)}
                    className="px-4 py-2 bg-zinc-700 hover:bg-zinc-600 rounded text-zinc-300 transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          )}

          {/* Lobby List */}
          <div>
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-semibold text-zinc-50">Available Games</h2>
              <button
                onClick={listLobbies}
                className="text-sm text-zinc-400 hover:text-zinc-300"
              >
                Refresh
              </button>
            </div>

            {(() => {
              // Filter out lobbies the player is already in
              const availableLobbies = lobbies.filter(
                (lobby) => !playerId || !lobby.players.some((p) => p.id === playerId)
              );
              
              if (availableLobbies.length === 0) {
                return (
                  <div className="text-center py-12 bg-zinc-800 rounded-lg">
                    <p className="text-zinc-400 mb-2">No games available</p>
                    <p className="text-zinc-500 text-sm">Create a new game to get started!</p>
                  </div>
                );
              }
              
              return (
                <div className="space-y-3">
                  {availableLobbies.map((lobby) => (
                  <div
                    key={lobby.id}
                    className="flex items-center justify-between p-4 bg-zinc-800 rounded-lg border border-zinc-700 hover:border-zinc-600 transition-colors"
                  >
                    <div>
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-zinc-50 font-medium">
                          {lobby.players[0]?.name}&apos;s Game
                        </span>
                        <span className="text-xs bg-zinc-700 px-2 py-1 rounded text-zinc-400">
                          {lobby.map_size}
                        </span>
                      </div>
                      <div className="flex items-center gap-4 text-sm text-zinc-400">
                        <span>{lobby.players.length}/{lobby.max_players} players</span>
                        <span>{MAP_SIZE_INFO[lobby.map_size].tiles} tiles</span>
                      </div>
                    </div>
                    <button
                      onClick={() => setJoinLobbyId(lobby.id)}
                      disabled={lobby.players.length >= lobby.max_players}
                      className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-zinc-600 disabled:cursor-not-allowed rounded text-white font-medium transition-colors"
                    >
                      {lobby.players.length >= lobby.max_players ? "Full" : "Join"}
                    </button>
                  </div>
                  ))}
                </div>
              );
            })()}
          </div>
        </div>
      </main>
    </div>
  );
}
