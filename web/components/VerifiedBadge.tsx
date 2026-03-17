import { Badge } from "@/components/ui/badge";

export function VerifiedBadge({ verified }: { verified: boolean }) {
  if (!verified) return null;
  return (
    <Badge variant="success" className="gap-1 text-[11px]">
      <svg className="h-3 w-3" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0zm3.78 4.78a.75.75 0 0 0-1.06-1.06L6.75 8.69 5.28 7.22a.75.75 0 0 0-1.06 1.06l2 2a.75.75 0 0 0 1.06 0l4.5-4.5z" />
      </svg>
      Verified
    </Badge>
  );
}
