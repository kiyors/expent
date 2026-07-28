import { buttonVariants } from "@expent/ui/components/button";
import { cn } from "@expent/ui/lib/utils";
import { Link, useLocation } from "@tanstack/react-router";

interface SidebarNavProps extends React.HTMLAttributes<HTMLElement> {
  items: {
    href: string;
    title: string;
    icon: React.ReactNode;
  }[];
}

export function SidebarNav({ className, items, ...props }: SidebarNavProps) {
  const pathname = useLocation().pathname;

  return (
    <nav className={cn("flex gap-x-2 lg:flex-col lg:gap-x-0 lg:gap-y-1", className)} {...props}>
      {items.map((item) => (
        <Link
          key={item.href}
          to={item.href as any}
          className={cn(
            buttonVariants({ variant: "ghost" }),
            pathname === item.href ? "bg-muted hover:bg-muted" : "hover:bg-transparent hover:underline",
            "justify-start gap-2",
          )}
        >
          {item.icon}
          {item.title}
        </Link>
      ))}
    </nav>
  );
}
