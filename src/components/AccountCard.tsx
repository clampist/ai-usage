import type { AccountStatus, UsageWindow } from "../types";
import { percentColor, toolBadge, useCountdown } from "../utils";

function UsageBar({ label, usedPercent, resetsAt, remainingLabel }: {
  label: string;
  usedPercent: number;
  resetsAt: string | null;
  remainingLabel?: string;
}) {
  const countdown = useCountdown(resetsAt);

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between text-[11px] text-white/70">
        <span>{label}</span>
        <span className="tabular-nums">{Math.round(usedPercent)}%</span>
      </div>
      <div className="h-1.5 w-full overflow-hidden rounded-full bg-white/10">
        <div
          className={`h-full rounded-full transition-all ${percentColor(usedPercent)}`}
          style={{ width: `${Math.min(100, usedPercent)}%` }}
        />
      </div>
      <div className="flex items-center justify-between text-[10px] text-white/45">
        <span>{countdown ?? "—"}</span>
        {remainingLabel && <span>{remainingLabel}</span>}
      </div>
    </div>
  );
}

export function AccountCard({
  account,
  showAccountEmail,
}: {
  account: AccountStatus;
  showAccountEmail: boolean;
}) {
  const staleMs = Date.now() - new Date(account.updated_at).getTime();
  const isStale = staleMs > 5 * 60 * 1000;

  const subtitleParts: string[] = [];
  if (account.plan) subtitleParts.push(account.plan);
  if (showAccountEmail && account.account_email) {
    subtitleParts.push(account.account_email);
  }
  const accountInfoLine =
    subtitleParts.length > 0 ? subtitleParts.join(" · ") : undefined;

  return (
    <div className="rounded-xl border border-white/10 bg-black/55 p-3 backdrop-blur-md">
      <div className="mb-2 flex items-start justify-between gap-2">
        <div className="min-w-0">
          <div className="truncate text-[12px] font-medium text-white">
            {account.display_name}
          </div>
          {accountInfoLine && (
            <div className="truncate text-[10px] text-white/50">
              {accountInfoLine}
            </div>
          )}
        </div>
        <span
          className={`shrink-0 rounded px-1.5 py-0.5 text-[9px] font-semibold uppercase tracking-wide ${toolBadge(account.tool)}`}
        >
          {account.tool}
        </span>
      </div>

      {account.ok && account.windows.length > 0 ? (
        <div className="space-y-2.5">
          {account.windows.map((w: UsageWindow) => (
            <UsageBar
              key={`${account.account_id}-${w.label}`}
              label={w.label}
              usedPercent={w.used_percent}
              resetsAt={w.resets_at}
              remainingLabel={w.remaining_label}
            />
          ))}
        </div>
      ) : (
        <div className="text-[11px] text-white/50">
          {account.error ?? "No usage data"}
        </div>
      )}

      {account.banked_resets != null && account.banked_resets > 0 && (
        <div className="mt-2 text-[10px] text-emerald-300">
          {account.banked_resets} banked reset{account.banked_resets === 1 ? "" : "s"}
        </div>
      )}

      {isStale && account.ok && (
        <div className="mt-1 text-[9px] text-amber-300/80">stale data</div>
      )}
    </div>
  );
}
