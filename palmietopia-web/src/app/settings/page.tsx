"use client";
import Link from "next/link";

export default function SettingsPage() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-zinc-900">
      <div className="text-center max-w-md">
        <h1 className="text-4xl font-bold text-zinc-50 mb-4">Settings</h1>
        <p className="text-zinc-400 mb-8">
          Settings page is coming soon! Customize your gameplay experience.
        </p>
        <div className="p-6 bg-zinc-800 rounded-lg border border-zinc-700 mb-8">
          <h2 className="text-lg font-semibold text-zinc-300 mb-2">Planned Settings</h2>
          <ul className="text-zinc-400 text-sm space-y-2 text-left">
            <li>• Audio volume controls</li>
            <li>• Graphics quality options</li>
            <li>• Keybindings customization</li>
            <li>• Accessibility options</li>
            <li>• Player profile settings</li>
          </ul>
        </div>
        <Link
          href="/"
          className="px-6 py-3 bg-zinc-700 hover:bg-zinc-600 rounded-lg text-zinc-50 font-medium transition-colors inline-block"
        >
          Back to Menu
        </Link>
      </div>
    </div>
  );
}
