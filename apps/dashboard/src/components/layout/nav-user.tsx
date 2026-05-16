"use client";

import { Avatar, AvatarFallback, AvatarImage } from "@expent/ui/components/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@expent/ui/components/dropdown-menu";
import { SidebarMenu, SidebarMenuButton, SidebarMenuItem, useSidebar } from "@expent/ui/components/sidebar";
import { BellIcon, LogOutIcon, MoreVerticalIcon, SettingsIcon, UserCogIcon } from "lucide-react";
import { useRouter } from "next/navigation";
import * as React from "react";
import { signOut, useSession } from "@/lib/auth-client";

export function NavUser() {
  const { isMobile } = useSidebar();
  const session = useSession();
  const router = useRouter();

  const user = session.data?.user ?? {
    name: "User",
    email: "",
    image: "",
  };

  const handleLogout = async () => {
    await signOut();
    window.location.href = "/sign-in";
  };

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <SidebarMenuButton
                size="lg"
                className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
              >
                <Avatar className="h-8 w-8 rounded-lg grayscale">
                  <AvatarImage src={user.image || ""} alt={user.name} />
                  <AvatarFallback className="rounded-lg">{user.name?.charAt(0) ?? "U"}</AvatarFallback>
                </Avatar>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-medium">{user.name}</span>
                  <span className="truncate text-xs text-muted-foreground">{user.email}</span>
                </div>
                <MoreVerticalIcon className="ml-auto size-4" />
              </SidebarMenuButton>
            }
          />
          <DropdownMenuContent
            className="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg"
            side={isMobile ? "bottom" : "right"}
            align="end"
            sideOffset={4}
          >
            <DropdownMenuGroup>
              <DropdownMenuLabel className="p-0 font-normal">
                <div className="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                  <Avatar className="h-8 w-8 rounded-lg">
                    <AvatarImage src={user.image || ""} alt={user.name} />
                    <AvatarFallback className="rounded-lg">{user.name?.charAt(0) ?? "U"}</AvatarFallback>
                  </Avatar>
                  <div className="grid flex-1 text-left text-sm leading-tight">
                    <span className="truncate font-medium">{user.name}</span>
                    <span className="truncate text-xs text-muted-foreground">{user.email}</span>
                  </div>
                </div>
              </DropdownMenuLabel>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem
                onClick={() => {
                  React.startTransition(() => {
                    if (typeof React.addTransitionType === "function") React.addTransitionType("nav-forward");
                    router.push("/settings/profile");
                  });
                }}
                className="cursor-pointer"
              >
                <UserCogIcon />
                Profile
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={() => {
                  React.startTransition(() => {
                    if (typeof React.addTransitionType === "function") React.addTransitionType("nav-forward");
                    router.push("/settings/account");
                  });
                }}
                className="cursor-pointer"
              >
                <SettingsIcon />
                Account
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={() => {
                  React.startTransition(() => {
                    if (typeof React.addTransitionType === "function") React.addTransitionType("nav-forward");
                    router.push("/settings/notifications");
                  });
                }}
                className="cursor-pointer"
              >
                <BellIcon />
                Notifications
              </DropdownMenuItem>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={handleLogout} className="cursor-pointer" variant="destructive">
              <LogOutIcon />
              Log out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
