"use client";
import Link from "next/link";

export default function Home() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-zinc-900">
      <div className="text-center mb-12">
        <h1 className="text-6xl font-bold text-zinc-50 mb-4">Palmietopia</h1>
        <p className="text-xl text-zinc-400">Turn-based strategy on a hexagonal world</p>
      </div>

      <div className="flex flex-col gap-4 w-80">
        <Link
          href="/singleplayer"
          className="px-8 py-4 bg-zinc-800 hover:bg-zinc-700 border border-zinc-600 rounded-lg text-zinc-50 text-xl font-semibold text-center transition-colors"
        >
          Singleplayer
        </Link>
        <Link
          href="/multiplayer"
          className="px-8 py-4 bg-emerald-700 hover:bg-emerald-600 border border-emerald-500 rounded-lg text-zinc-50 text-xl font-semibold text-center transition-colors"
        >
          Multiplayer
        </Link>
        <Link
          href="/settings"
          className="px-8 py-4 bg-zinc-800 hover:bg-zinc-700 border border-zinc-600 rounded-lg text-zinc-50 text-xl font-semibold text-center transition-colors"
        >
          Settings
        </Link>
      </div>

      <p className="mt-12 text-zinc-500 text-sm">Up to 5 players • Multiple map sizes</p>
    </div>
  );
}
