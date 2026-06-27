import { useEffect, useState } from "react";

export function useCountdown(resetsAt: string | null): string | null {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    if (!resetsAt) {
      setLabel(null);
      return;
    }

    const tick = () => {
      const target = new Date(resetsAt).getTime();
      const diff = target - Date.now();
      if (Number.isNaN(target) || diff <= 0) {
        setLabel("resetting soon");
        return;
      }
      const totalSec = Math.floor(diff / 1000);
      const days = Math.floor(totalSec / 86400);
      const hours = Math.floor((totalSec % 86400) / 3600);
      const mins = Math.floor((totalSec % 3600) / 60);
      if (days > 0) setLabel(`resets ${days}d ${hours}h`);
      else if (hours > 0) setLabel(`resets ${hours}h ${mins}m`);
      else setLabel(`resets ${mins}m`);
    };

    tick();
    const id = window.setInterval(tick, 1000);
    return () => window.clearInterval(id);
  }, [resetsAt]);

  return label;
}

export function percentColor(percent: number): string {
  if (percent >= 85) return "bg-red-500";
  if (percent >= 70) return "bg-amber-400";
  return "bg-emerald-400";
}

export function toolBadge(tool: string): string {
  switch (tool) {
    case "claude":
      return "bg-orange-500/20 text-orange-200";
    case "codex":
      return "bg-green-500/20 text-green-200";
    case "cursor":
      return "bg-blue-500/20 text-blue-200";
    case "antigravity":
      return "bg-violet-500/20 text-violet-200";
    default:
      return "bg-white/10 text-white/70";
  }
}
