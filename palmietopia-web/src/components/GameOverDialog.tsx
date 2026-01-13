interface GameOverDialogProps {
  isWinner: boolean;
  winnerName: string | null;
  onClose: () => void;
}

export function GameOverDialog({ isWinner, winnerName, onClose }: GameOverDialogProps) {
  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-zinc-800 rounded-lg p-8 text-center shadow-2xl border border-zinc-700 min-w-80">
        <h1 className={`text-4xl font-bold mb-4 ${isWinner ? "text-emerald-400" : "text-red-400"}`}>
          {isWinner ? "Victory!" : "Defeat"}
        </h1>
        <p className="text-xl text-zinc-300 mb-8">
          {isWinner ? "You have conquered all opponents!" : `${winnerName} has won the game!`}
        </p>
        <button
          onClick={onClose}
          className="px-6 py-3 bg-emerald-600 hover:bg-emerald-500 rounded text-white font-medium transition-colors"
        >
          Back to Lobby
        </button>
      </div>
    </div>
  );
}
