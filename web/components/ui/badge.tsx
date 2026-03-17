import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors",
  {
    variants: {
      variant: {
        default: "border-transparent bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900",
        secondary: "border-transparent bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100",
        success: "border-transparent bg-green-100 dark:bg-green-900/40 text-green-800 dark:text-green-400",
        warning: "border-transparent bg-yellow-100 dark:bg-yellow-900/40 text-yellow-800 dark:text-yellow-400",
        outline: "border-zinc-200 dark:border-zinc-700 text-zinc-700 dark:text-zinc-300",
      },
    },
    defaultVariants: { variant: "default" },
  }
);

interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

export function Badge({ className, variant, ...props }: BadgeProps) {
  return <div className={cn(badgeVariants({ variant }), className)} {...props} />;
}
