"use client";
import { useCallback, useEffect, useRef, useState } from "react";

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
  player_times_ms: number[];  // Time bank for each player
  turn_started_at_ms: number;
  base_time_ms: number;
  increment_ms: number;
}

export type ServerMessage =
  | { type: "LobbyCreated"; lobby_id: string; player_id: string }
  | { type: "JoinedLobby"; lobby: Lobby; player_id: string }
  | { type: "LobbyUpdated"; lobby: Lobby }
  | { type: "LobbyList"; lobbies: Lobby[] }
  | { type: "GameStarted"; game: GameSession }
  | { type: "GameRejoined"; game: GameSession }
  | { type: "PlayerLeft"; player_id: string }
  | { type: "Error"; message: string }
  | { type: "TurnChanged"; current_turn: number; player_times_ms: number[] }
  | { type: "TimeTick"; player_index: number; remaining_ms: number };

export type ClientMessage =
  | { type: "CreateLobby"; player_name: string; map_size: MapSize }
  | { type: "JoinLobby"; lobby_id: string; player_name: string }
  | { type: "LeaveLobby" }
  | { type: "StartGame" }
  | { type: "ListLobbies" }
  | { type: "EndTurn"; game_id: string; player_id: string }
  | { type: "RejoinGame"; game_id: string; player_id: string };

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001/ws";

export function useWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [playerId, setPlayerId] = useState<string | null>(null);
  const [currentLobby, setCurrentLobby] = useState<Lobby | null>(null);
  const [lobbies, setLobbies] = useState<Lobby[]>([]);
  const [game, setGame] = useState<GameSession | null>(null);
  const [turnTimeRemaining, setTurnTimeRemaining] = useState<number>(0);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const playerIdRef = useRef<string | null>(null); // Ref to avoid stale closure

  // Auto-connect on mount, cleanup on unmount
  useEffect(() => {
    const ws = new WebSocket(WS_URL);
    wsRef.current = ws;

    ws.onopen = () => {
      setIsConnected(true);
      setError(null);
      ws.send(JSON.stringify({ type: "ListLobbies" }));
    };

    ws.onclose = () => {
      setIsConnected(false);
    };

    ws.onerror = () => {
      setError("Connection error");
    };

    ws.onmessage = (event) => {
      try {
        const msg: ServerMessage = JSON.parse(event.data);
        
        switch (msg.type) {
          case "LobbyCreated":
            setPlayerId(msg.player_id);
            playerIdRef.current = msg.player_id; // Update ref synchronously
            break;
          case "JoinedLobby":
            setPlayerId(msg.player_id);
            playerIdRef.current = msg.player_id; // Update ref synchronously
            setCurrentLobby(msg.lobby);
            break;
          case "LobbyUpdated":
            setCurrentLobby(msg.lobby);
            break;
          case "LobbyList":
            setLobbies(msg.lobbies);
            break;
          case "GameStarted":
            console.log("GameStarted received:", msg.game);
            setGame(msg.game);
            // Set initial time for current player
            setTurnTimeRemaining(msg.game.player_times_ms[msg.game.current_turn]);
            if (typeof window !== "undefined") {
              sessionStorage.setItem(`game-${msg.game.id}`, JSON.stringify(msg.game));
              // Store player ID so game page knows which player we are
              const currentPlayerId = playerIdRef.current;
              if (currentPlayerId) {
                sessionStorage.setItem(`player-${msg.game.id}`, currentPlayerId);
              }
            }
            break;
          case "GameRejoined":
            console.log("GameRejoined received:", msg.game);
            setGame(msg.game);
            setTurnTimeRemaining(msg.game.player_times_ms[msg.game.current_turn]);
            break;
          case "TurnChanged":
            console.log("TurnChanged received:", msg);
            setGame((prev) =>
              prev ? { 
                ...prev, 
                current_turn: msg.current_turn,
                player_times_ms: msg.player_times_ms,
                turn_started_at_ms: Date.now(), // Reset timer start
              } : null
            );
            // Set time for new current player
            setTurnTimeRemaining(msg.player_times_ms[msg.current_turn]);
            break;
          case "TimeTick":
            // Only update if this is for the current player
            setGame((prev) => {
              if (prev && prev.current_turn === msg.player_index) {
                setTurnTimeRemaining(msg.remaining_ms);
              }
              return prev;
            });
            break;
          case "PlayerLeft":
            break;
          case "Error":
            setError(msg.message);
            break;
        }
      } catch (e) {
        console.error("Failed to parse message:", e);
      }
    };

    return () => {
      ws.close();
    };
  }, []);

  const send = useCallback((msg: ClientMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(msg));
    }
  }, []);

  const createLobby = useCallback(
    (playerName: string, mapSize: MapSize) => {
      send({ type: "CreateLobby", player_name: playerName, map_size: mapSize });
    },
    [send]
  );

  const joinLobby = useCallback(
    (lobbyId: string, playerName: string) => {
      send({ type: "JoinLobby", lobby_id: lobbyId, player_name: playerName });
    },
    [send]
  );

  const leaveLobby = useCallback(() => {
    send({ type: "LeaveLobby" });
    setCurrentLobby(null);
  }, [send]);

  const startGame = useCallback(() => {
    send({ type: "StartGame" });
  }, [send]);

  const listLobbies = useCallback(() => {
    send({ type: "ListLobbies" });
  }, [send]);

  const endTurn = useCallback((gameId: string, playerId: string) => {
    console.log("Sending EndTurn:", { gameId, playerId });
    send({ type: "EndTurn", game_id: gameId, player_id: playerId });
  }, [send]);

  const rejoinGame = useCallback((gameId: string, playerId: string) => {
    console.log("Sending RejoinGame:", { gameId, playerId });
    send({ type: "RejoinGame", game_id: gameId, player_id: playerId });
  }, [send]);

  return {
    isConnected,
    playerId,
    currentLobby,
    lobbies,
    game,
    turnTimeRemaining,
    error,
    createLobby,
    joinLobby,
    leaveLobby,
    startGame,
    listLobbies,
    endTurn,
    rejoinGame,
    setError,
    setCurrentLobby,
    setGame,
  };
}
