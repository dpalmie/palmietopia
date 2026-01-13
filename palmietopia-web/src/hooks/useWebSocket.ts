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
}

export type ServerMessage =
  | { type: "LobbyCreated"; lobby_id: string; player_id: string }
  | { type: "LobbyUpdated"; lobby: Lobby }
  | { type: "LobbyList"; lobbies: Lobby[] }
  | { type: "GameStarted"; game: GameSession }
  | { type: "PlayerLeft"; player_id: string }
  | { type: "Error"; message: string };

export type ClientMessage =
  | { type: "CreateLobby"; player_name: string; map_size: MapSize }
  | { type: "JoinLobby"; lobby_id: string; player_name: string }
  | { type: "LeaveLobby" }
  | { type: "StartGame" }
  | { type: "ListLobbies" };

interface UseWebSocketOptions {
  onMessage?: (msg: ServerMessage) => void;
  onOpen?: () => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
}

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:3001/ws";

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const [isConnected, setIsConnected] = useState(false);
  const [playerId, setPlayerId] = useState<string | null>(null);
  const [currentLobby, setCurrentLobby] = useState<Lobby | null>(null);
  const [lobbies, setLobbies] = useState<Lobby[]>([]);
  const [game, setGame] = useState<GameSession | null>(null);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const ws = new WebSocket(WS_URL);

    ws.onopen = () => {
      setIsConnected(true);
      setError(null);
      options.onOpen?.();
    };

    ws.onclose = () => {
      setIsConnected(false);
      options.onClose?.();
      // Auto-reconnect after 3 seconds
      reconnectTimeoutRef.current = setTimeout(() => {
        connect();
      }, 3000);
    };

    ws.onerror = (e) => {
      setError("Connection error");
      options.onError?.(e);
    };

    ws.onmessage = (event) => {
      try {
        const msg: ServerMessage = JSON.parse(event.data);
        handleMessage(msg);
        options.onMessage?.(msg);
      } catch (e) {
        console.error("Failed to parse message:", e);
      }
    };

    wsRef.current = ws;
  }, [options]);

  const handleMessage = useCallback((msg: ServerMessage) => {
    switch (msg.type) {
      case "LobbyCreated":
        setPlayerId(msg.player_id);
        break;
      case "LobbyUpdated":
        setCurrentLobby(msg.lobby);
        break;
      case "LobbyList":
        setLobbies(msg.lobbies);
        break;
      case "GameStarted":
        setGame(msg.game);
        // Store game in sessionStorage for the game page
        if (typeof window !== "undefined") {
          sessionStorage.setItem(`game-${msg.game.id}`, JSON.stringify(msg.game));
        }
        break;
      case "PlayerLeft":
        // Handled by LobbyUpdated
        break;
      case "Error":
        setError(msg.message);
        break;
    }
  }, []);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    wsRef.current?.close();
    wsRef.current = null;
    setIsConnected(false);
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

  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  return {
    isConnected,
    playerId,
    currentLobby,
    lobbies,
    game,
    error,
    connect,
    disconnect,
    createLobby,
    joinLobby,
    leaveLobby,
    startGame,
    listLobbies,
    setError,
    setCurrentLobby,
    setGame,
  };
}
