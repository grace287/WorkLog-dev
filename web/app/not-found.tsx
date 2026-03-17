import Link from "next/link";

export default function NotFound() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-4 text-center px-4">
      <h1 className="text-6xl font-bold text-zinc-200">404</h1>
      <h2 className="text-xl font-semibold text-zinc-800">Portfolio not found</h2>
      <p className="text-zinc-500 max-w-sm">
        This portfolio doesn&apos;t exist or has been set to private.
      </p>
      <Link
        href="/"
        className="mt-2 rounded-lg bg-zinc-900 px-4 py-2 text-sm text-white hover:bg-zinc-700 transition-colors"
      >
        Go home
      </Link>
    </main>
  );
}
