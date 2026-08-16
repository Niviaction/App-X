import Link from "next/link";

export default function Home() {
  return (
    <div className="flex flex-col flex-1 items-center justify-center gap-4 bg-zinc-50 font-sans dark:bg-black">
      <h1 className="text-2xl font-semibold">App X</h1>
      <div className="flex gap-3">
        <Link href="/login" className="rounded-md bg-blue-600 px-4 py-2 text-white">
          Log In
        </Link>
        <Link href="/signup" className="rounded-md border px-4 py-2">
          Sign Up
        </Link>
      </div>
    </div>
  );
}
