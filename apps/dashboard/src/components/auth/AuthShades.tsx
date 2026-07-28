export interface AuthShadesProps {
  variant?: "default" | "flipped";
}

export function AuthShades({ variant = "default" }: AuthShadesProps) {
  const isFlipped = variant === "flipped";

  return (
    <div aria-hidden className="absolute inset-0 isolate -z-10 opacity-60 contain-strict">
      {/* Top blob */}
      <div
        className={`absolute top-0 h-320 w-140 -translate-y-87.5 rounded-full bg-[radial-gradient(68.54%_68.72%_at_55.02%_31.46%,--theme(--color-foreground/.06)_0,hsla(0,0%,55%,.02)_50%,--theme(--color-foreground/.01)_80%)] ${
          isFlipped ? "left-0 translate-x-[-20%]" : "right-0"
        }`}
      />
      {/* Middle blob */}
      <div
        className={`absolute top-0 h-320 w-60 rounded-full bg-[radial-gradient(50%_50%_at_50%_50%,--theme(--color-foreground/.04)_0,--theme(--color-foreground/.01)_80%,transparent_100%)] [translate:5%_-50%] ${
          isFlipped ? "left-0 -translate-x-1/2" : "right-0"
        }`}
      />
      {/* Bottom blob */}
      <div
        className={`absolute top-0 h-320 w-60 -translate-y-87.5 rounded-full bg-[radial-gradient(50%_50%_at_50%_50%,--theme(--color-foreground/.04)_0,--theme(--color-foreground/.01)_80%,transparent_100%)] ${
          isFlipped ? "left-0 -translate-x-1/4" : "right-0"
        }`}
      />
    </div>
  );
}
